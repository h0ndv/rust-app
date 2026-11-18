import { invoke } from '@tauri-apps/api/core';
import { useState } from 'react';

function App() {
    const [status, setStatus] = useState('');

    async function open(command: string) {
        setStatus('Ejecutando');
        try {
            const result = await invoke<string | void>(command);
            setStatus(typeof result === 'string' ? result : 'Ventana lista');
        } catch (error) {
            setStatus(String(error));
        }
    }

    return (
        <main className="flex min-h-svh flex-col items-center justify-center gap-2 bg-background p-8 text-foreground">
            <h1 className="text-2xl font-semibold tracking-tight">rust-app</h1>
            <p className="text-sm text-muted-foreground">
                Sin datos todavía. Falta conectar la cuenta y sincronizar.
            </p>

            {/* Arnés temporal de la Task 4. Lo reemplaza el onboarding en la Task 11. */}
            <div className="mt-6 flex flex-col items-center gap-2 rounded-lg border border-border p-4">
                <p className="text-xs tracking-wide text-muted-foreground uppercase">
                    Prueba de ventanas
                </p>
                <div className="flex gap-2">
                    <button
                        type="button"
                        className="rounded-md border border-border px-3 py-1 text-sm hover:bg-muted"
                        onClick={() => open('open_login')}
                    >
                        Login
                    </button>
                    <button
                        type="button"
                        className="rounded-md border border-border px-3 py-1 text-sm hover:bg-muted"
                        onClick={() => open('open_moodle')}
                    >
                        Moodle
                    </button>
                    <button
                        type="button"
                        className="rounded-md border border-border px-3 py-1 text-sm hover:bg-muted"
                        onClick={() => open('open_banner')}
                    >
                        Banner
                    </button>
                </div>
                <p className="text-xs tracking-wide text-muted-foreground uppercase">
                    Prueba de clientes
                </p>
                <div className="flex gap-2">
                    <button
                        type="button"
                        className="rounded-md border border-border px-3 py-1 text-sm hover:bg-muted"
                        onClick={() => open('smoke_moodle')}
                    >
                        API Moodle
                    </button>
                    <button
                        type="button"
                        className="rounded-md border border-border px-3 py-1 text-sm hover:bg-muted"
                        onClick={() => open('smoke_ellucian')}
                    >
                        API Ellucian
                    </button>
                    <button
                        type="button"
                        className="rounded-md border border-border px-3 py-1 text-sm hover:bg-muted"
                        onClick={() => open('smoke_banner')}
                    >
                        API Banner
                    </button>
                </div>
                <button
                    type="button"
                    className="rounded-md border border-border px-3 py-1 text-sm hover:bg-muted"
                    onClick={() => open('smoke_sync')}
                >
                    Sincronizar
                </button>
                {status ? (
                    <p className="text-xs text-muted-foreground">{status}</p>
                ) : null}
            </div>
        </main>
    );
}

export default App;
