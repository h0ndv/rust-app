import { Link, useRouterState } from '@tanstack/react-router';
import {
    BookOpen,
    CalendarDays,
    ClipboardList,
    FolderOpen,
    GraduationCap,
    Home,
    Newspaper,
    Search,
    Settings,
} from 'lucide-react';
import { useState, type ReactNode } from 'react';

import { Badge } from '@/components/ui/badge';
import { Separator } from '@/components/ui/separator';
import {
    Sidebar,
    SidebarContent,
    SidebarFooter,
    SidebarGroup,
    SidebarGroupContent,
    SidebarGroupLabel,
    SidebarHeader,
    SidebarInset,
    SidebarMenu,
    SidebarMenuButton,
    SidebarMenuItem,
    SidebarProvider,
    SidebarTrigger,
} from '@/components/ui/sidebar';
import { api } from '@/lib/api';

const NAV = [
    { to: '/', label: 'Inicio', icon: Home },
    { to: '/ramos', label: 'Ramos', icon: BookOpen },
    { to: '/tareas', label: 'Tareas', icon: ClipboardList },
    { to: '/recursos', label: 'Recursos', icon: FolderOpen },
    { to: '/notas', label: 'Notas', icon: GraduationCap },
    { to: '/horario', label: 'Horario', icon: CalendarDays },
    { to: '/novedades', label: 'Novedades', icon: Newspaper },
    { to: '/buscar', label: 'Buscador', icon: Search },
    { to: '/ajustes', label: 'Ajustes', icon: Settings },
];

interface AppShellProps {
    children: ReactNode;
}

const LABELS: Record<string, string> = {
    ok: 'Al día',
    partial: 'Sincronización parcial',
    failed: 'Falló la sincronización',
    running: 'Sincronizando',
};

export function AppShell({ children }: AppShellProps) {
    const pathname = useRouterState({
        select: (state) => state.location.pathname,
    });
    const [syncing, setSyncing] = useState(false);
    const [syncLabel, setSyncLabel] = useState('Sin sincronizar');
    const [lastSync, setLastSync] = useState<string | null>(null);

    async function sync(): Promise<void> {
        setSyncing(true);
        setSyncLabel(LABELS.running);

        try {
            const run = await api.startSync();
            setSyncLabel(LABELS[run.status] ?? 'Sincronizado');
            setLastSync(new Date().toLocaleTimeString('es-CL'));
        } catch (cause) {
            setSyncLabel(String(cause));
        } finally {
            setSyncing(false);
        }
    }

    return (
        <SidebarProvider>
            <Sidebar>
                <SidebarHeader>
                    <div className="px-2 py-1 text-sm font-semibold tracking-tight text-primary">
                        rust-app
                    </div>
                </SidebarHeader>

                <SidebarContent>
                    <SidebarGroup>
                        <SidebarGroupLabel>Navegación</SidebarGroupLabel>
                        <SidebarGroupContent>
                            <SidebarMenu>
                                {NAV.map((item) => (
                                    <SidebarMenuItem key={item.to}>
                                        <SidebarMenuButton
                                            isActive={pathname === item.to}
                                            tooltip={item.label}
                                            render={<Link to={item.to} />}
                                        >
                                            <item.icon />
                                            <span>{item.label}</span>
                                        </SidebarMenuButton>
                                    </SidebarMenuItem>
                                ))}
                            </SidebarMenu>
                        </SidebarGroupContent>
                    </SidebarGroup>
                </SidebarContent>

                <SidebarFooter>
                    <div className="px-2 py-1 text-xs text-muted-foreground">
                        Cuenta
                    </div>
                </SidebarFooter>
            </Sidebar>

            <SidebarInset>
                <header className="flex h-14 items-center gap-3 border-b border-border px-4">
                    <SidebarTrigger />
                    <Separator orientation="vertical" className="h-6" />
                    <Badge variant={syncing ? 'secondary' : 'outline'}>
                        {syncLabel}
                    </Badge>
                    {lastSync ? (
                        <span className="text-xs text-muted-foreground">
                            Última actualización: {lastSync}
                        </span>
                    ) : null}
                    <div className="ml-auto flex items-center gap-2">
                        <button
                            type="button"
                            className="rounded-md border border-border px-3 py-1 text-sm hover:bg-muted disabled:opacity-50"
                            onClick={sync}
                            disabled={syncing}
                        >
                            {syncing ? 'Sincronizando' : 'Sincronizar'}
                        </button>
                    </div>
                </header>

                <div className="flex-1 overflow-auto p-6">{children}</div>
            </SidebarInset>
        </SidebarProvider>
    );
}
