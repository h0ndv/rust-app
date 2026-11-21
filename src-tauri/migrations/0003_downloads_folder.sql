-- The downloads folder was renamed from PanelRamos to rust-app.
-- Only the stored setting is updated; already downloaded files keep their path.

UPDATE settings
SET value = replace(value, '\Downloads\PanelRamos', '\Downloads\rust-app')
WHERE key = 'downloads_dir'
  AND value LIKE '%\Downloads\PanelRamos';

UPDATE settings
SET value = replace(value, '/Downloads/PanelRamos', '/Downloads/rust-app')
WHERE key = 'downloads_dir'
  AND value LIKE '%/Downloads/PanelRamos';
