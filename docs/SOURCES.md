# Data sources: verified contracts

Everything here was captured from the real account through the app's own WebView2
session and replayed with `requests`. Nothing is inferred from documentation.

Re-check these notes whenever an adapter starts returning empty data. The capture
recipe at the end explains how to reproduce it.

## 1. Moodle

Base: `https://aulasvirtuales.santotomas.cl`
Cookie: `MoodleSession` (session cookie, dies with the process)

### AJAX envelope

All JSON calls go through:

```http
POST /lib/ajax/service.php?sesskey=<key>&info=<method>
Content-Type: application/json

[{"index": 0, "methodname": "<method>", "args": {...}}]
```

`sesskey` lives in `M.cfg` on `/my/courses.php` and is cached in memory per client.

**Methods used**

| Method                                                        | Args                                                                        | Result                                                                                           |
| ------------------------------------------------------------- | --------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `core_course_get_enrolled_courses_by_timeline_classification` | `{offset, limit, classification: "all", sort: "fullname", …}`               | `data.courses[]` with `id`, `fullname`, `shortname`, `coursecategory`                            |
| `core_calendar_get_action_events_by_timesort`                 | `{limitnum: 50, timesortfrom, timesortto, limittononsuspendedevents: true}` | `data.events[]` with `name`, `modulename`, `instance` (cmid), `timesort`, `overdue`, `course.id` |

**Errors** arrive as `[{"error": true, "exception": {"errorcode": "requirelogin"}}]`.
Codes `requirelogin`, `invalidsesskey`, `sessionexpired`, `invalidlogin` mean the
session is gone and map to `Unauthorized`.

### HTML pages

| Page                                       | What to read                                                                                         |
| ------------------------------------------ | ---------------------------------------------------------------------------------------------------- |
| `GET /course/view.php?id=<id>`             | Section names in `a[href*="#tabs-tree-start"]` (`title` attribute); section numbers from `section=N` |
| `GET /course/view.php?id=<id>&section=<n>` | One section's activities                                                                             |
| `GET /mod/assign/view.php?id=<cmid>`       | Submission status, dates, description, attachments                                                   |

**Activity list**: `li.activity` elements with `id="module-<cmid>"`, a `modtype_*` class
and a `data-activityname` attribute holding the visible name. There is **no
`a.aalink`** in this Moodle 4.5 theme; the link is a plain `a[href]`.

**Assignment status**: the container is a `div.submissionstatustable` (not a table),
with rows "Estado de la entrega" and "Estado de la calificación". Texts seen:
"Todavía no se han realizado envíos" and "Sin calificar".

**Dates**: `[data-region='activity-dates']` with
`Apertura: miércoles, 2 de septiembre de 2026, 00:00`, plus `Cierre` and `Corte`.

**Attachments**: `a[href*="pluginfile.php"]`, excluding `/theme_` (the favicon appears
on every page).

**Course total**: `GET /grade/report/user/index.php?id=<id>` returns **404** in this
installation. The overview report has only `-` in every row, so Moodle totals are
currently `None`.

**Content types matter**: a `mod/resource/view.php` URL redirects to the PDF. Reading
that body as text stalls the sync, so resources are stored as files without fetching
the page.

## 2. Ellucian Experience

Endpoint: `POST https://experience.elluciancloud.com/ustc/api/ethos-query`
Cookie: `ustc-exp.sid`, plus `Origin` and `Referer` headers.

**It is not GraphQL.** The body is a query descriptor; dropping any of the first four
fields returns `400` with an empty body.

```json
{
    "extensionId": "6f147f85-1a0f-4d88-884c-ada96a44bdb9",
    "cardId": "001G000000oShyJIAS|Ellucian-LAC|Clases|Clases",
    "manifestType": "Clases",
    "queryId": "<query>",
    "properties": {}
}
```

| `queryId`                                  | Properties                       | Shape                                                                                                                                     |
| ------------------------------------------ | -------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| `student-academic-periods`                 | `{}`                             | `data.sectionRegistrations.edges[].node.section.reportingAcademicPeriod`; repeated per historical section, dedupe by `id`                 |
| `section-registrations-by-academic-period` | `{"academicPeriodId": "<uuid>"}` | `…node.section` with `id`, `code` (the CRN), `course.subject.abbreviation`, `course.number`, `course.titles[]`, `reportingAcademicPeriod` |
| `instructional-events-by-section`          | `{"sectionId": "<uuid>"}`        | `data.instructionalEvents.edges[].node`                                                                                                   |
| `student-transcript-grades`                | `{"sectionId": "<uuid>"}`        | `data.studentTranscriptGrades`; empty today                                                                                               |

**Schedule node**

```json
{
    "id": "7679d0c6-…",
    "recurrence": {
        "timePeriod": {
            "startOn": "2026-08-10T19:21:00-04:00",
            "endOn": "2026-12-31T20:00:00-03:00"
        },
        "repeatRule": { "type": "weekly", "daysOfWeek": ["friday"] }
    },
    "locations": [
        {
            "location": {
                "room": {
                    "number": "S-01-P04-D",
                    "title": "Sala 401",
                    "building": {
                        "title": "Dieciocho N° 1191",
                        "code": "AR0004"
                    }
                }
            }
        }
    ],
    "instructorsInfo": { "firstName": "PABLO", "lastName": "LILLO;GALLARDO" }
}
```

`startOn` carries the first date plus the start time; `endOn` carries the term end plus
the end time. Only the time-of-day is used. `lastName` can hold two surnames separated
by `;`.

**The dashboard card "Horario Semanal" is locked and never loads.** The underlying
query works when called per section, which is what the adapter does.

## 3. Banner 9

Base: `https://selferviceprd.santotomas.cl/StudentSelfService/studentGrades`
**Without `/ssb/`**, although the original spike documented that path.
Cookie: `JSESSIONID`.

| Endpoint                                                                         | Result                                                                                                  |
| -------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------- |
| `GET /term`                                                                      | `[{"code": "202620", "description": "Semestre Pregrado 2026 - 2"}]`; the first row is the `-1` wildcard |
| `GET /level?term=CODE`                                                           | `[{"code": "02", "description": "Profesional"}]`                                                        |
| `GET /studyPath?term=CODE&level=CODE`                                            | Includes `-1` and `-2` ("Todos los cursos")                                                             |
| `GET /courses?termCode=&levelCode=&studyPathCode=-2&pageOffset=0&pageMaxSize=25` | `{success, data[], totalCount}`                                                                         |
| `GET /gpa?term=CODE`                                                             | `{selectedTerm: {gpa}, cumulative: {gpa}}`                                                              |

**Grade row**

```json
{
    "subjectCode": "IGR",
    "courseNumber": "00167",
    "courseReferenceNumber": "30061",
    "section": "00G",
    "courseTitle": "Minería De Datos",
    "finalGrade": null,
    "midtermGrade": null,
    "historyFinalGrade": null,
    "calculatedFinalGrade": null,
    "termCode": "202620",
    "levelCode": "02"
}
```

**`reqwest` limitation**: `/term`, `/level` and `/studyPath` respond with the Microsoft
sign-in page when called outside the browser. `/courses` and `/gpa` only work inside
the page. The client tries `reqwest` first and falls back to `eval_json` in a hidden
window that shares the WebView2 profile.

**Grades today**: no grade is published; all eight rows have every grade field in
`null`.

## 4. How the app links Moodle and Banner

- **CRN**: Moodle shortname prefix (`30061.202620` → `30061`) equals Banner
  `courseReferenceNumber`.
- **Code and section**: `(IGR-00167-00G)` in the Moodle full name equals
  `subjectCode`, `courseNumber` and `section`.
- The term must match when both sides have one. Sections `A0C` (lecture) and `00G`
  (lab) of `IGR-00167` prove both paths work.

## 5. How to re-capture a contract

The WebView2 profile holds the session, so the capture runs against the app itself.

```powershell
$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=9333 --remote-allow-origins=*"
bun run tauri dev
```

Then, from Python:

1. `GET http://127.0.0.1:9333/json/list` lists page targets.
2. Connect to `webSocketDebuggerUrl`.
3. `Network.enable` to see requests, or `Runtime.evaluate` to run `fetch` inside the
   authenticated page, which is the only way to call the Banner endpoints.

`Runtime.evaluate` with `awaitPromise: true` and a script that returns
`JSON.stringify({status, body})` is enough to save a fixture.

Never write cookies to disk. Read them, use them in memory and drop them.

## 6. Fixture hygiene

Fixtures are committed because the tests read them with `include_str!`: without them a
fresh clone does not even compile the test suite. They are not ignored, but they are
cleaned before being committed:

- Only the files a test actually uses are kept. Anything else is deleted; it can be
  re-captured with the recipe above.
- The student's name becomes `Estudiante de Prueba`, the profile id becomes `1` and
  every `sesskey` becomes `sesskey=0000000000`.
- Large JSON files are trimmed to the events the test needs.

Current size: about 0.7 MB for twelve files.
