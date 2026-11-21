import { useQuery } from '@tanstack/react-query';
import { useEffect, useState } from 'react';

import { api, type SettingsDto } from '@/lib/api';

import { Loading, Screen } from '../common/Screen';

const THEMES = [
    { value: 'system', label: 'Sistema' },
    { value: 'light', label: 'Claro' },
    { value: 'dark', label: 'Oscuro' },
];

export function SettingsScreen() {
    const { data } = useQuery({
        queryKey: ['settings'],
        queryFn: api.getSettings,
    });

    const [form, setForm] = useState<SettingsDto | null>(null);
    const [status, setStatus] = useState('');

    useEffect(() => {
        if (data && form === null) {
            setForm(data);
        }
    }, [data, form]);

    async function save(): Promise<void> {
        if (!form) {
            return;
        }
        try {
            await api.saveSettings(form);
            setStatus('Ajustes guardados.');
        } catch (cause) {
            setStatus(String(cause));
        }
    }

    async function clear(): Promise<void> {
        if (!window.confirm('Se borran los datos locales. ¿Continuar?')) {
            return;
        }
        try {
            const deleted = await api.clearData();
            setStatus(`${deleted} filas borradas.`);
        } catch (cause) {
            setStatus(String(cause));
        }
    }

    if (!form) {
        return (
            <Screen title="Ajustes">
                <Loading />
            </Screen>
        );
    }

    return (
        <Screen title="Ajustes" subtitle="Todo se guarda en este computador.">
            <div className="flex max-w-md flex-col gap-4">
                <label className="flex flex-col gap-1 text-sm">
                    <span className="font-medium">
                        Intervalo de sincronización (minutos)
                    </span>
                    <input
                        type="number"
                        min={5}
                        className="rounded-md border border-border bg-background px-3 py-2"
                        value={form.sync_interval_minutes}
                        onChange={(event) =>
                            setForm({
                                ...form,
                                sync_interval_minutes: Number(
                                    event.target.value,
                                ),
                            })
                        }
                    />
                </label>

                <label className="flex flex-col gap-1 text-sm">
                    <span className="font-medium">Carpeta de descargas</span>
                    <input
                        type="text"
                        className="rounded-md border border-border bg-background px-3 py-2"
                        value={form.downloads_dir}
                        onChange={(event) =>
                            setForm({
                                ...form,
                                downloads_dir: event.target.value,
                            })
                        }
                    />
                </label>

                <label className="flex flex-col gap-1 text-sm">
                    <span className="font-medium">Tema</span>
                    <select
                        className="rounded-md border border-border bg-background px-3 py-2"
                        value={form.theme}
                        onChange={(event) =>
                            setForm({ ...form, theme: event.target.value })
                        }
                    >
                        {THEMES.map((theme) => (
                            <option key={theme.value} value={theme.value}>
                                {theme.label}
                            </option>
                        ))}
                    </select>
                </label>

                <label className="flex items-center gap-2 text-sm">
                    <input
                        type="checkbox"
                        checked={form.notifications}
                        onChange={(event) =>
                            setForm({
                                ...form,
                                notifications: event.target.checked,
                            })
                        }
                    />
                    <span className="font-medium">Avisar de vencimientos</span>
                </label>

                <div className="flex flex-wrap gap-2">
                    <button
                        type="button"
                        className="rounded-md bg-primary px-4 py-2 text-sm text-primary-foreground"
                        onClick={save}
                    >
                        Guardar
                    </button>
                </div>

                <section className="mt-4 flex flex-col gap-2 rounded-lg border border-destructive/40 p-4">
                    <h3 className="text-sm font-semibold">Datos locales</h3>
                    <p className="text-xs text-muted-foreground">
                        Borra ramos, tareas, recursos y notas de este
                        computador. El perfil de sesión no se toca.
                    </p>
                    <button
                        type="button"
                        className="self-start rounded-md border border-destructive px-3 py-1 text-sm text-destructive"
                        onClick={clear}
                    >
                        Borrar datos
                    </button>
                </section>

                {status ? (
                    <p className="text-sm text-muted-foreground">{status}</p>
                ) : null}
            </div>
        </Screen>
    );
}
