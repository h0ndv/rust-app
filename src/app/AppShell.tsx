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
import type { ReactNode } from 'react';
import { useTranslation } from 'react-i18next';

import { Progress } from '@/components/ui/progress';
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
import { LANGUAGES } from '@/i18n';

import { SyncBadge } from './SyncBadge';
import { useSyncController } from './use-sync';

const NAV = [
    { to: '/', labelKey: 'nav.home' as const, icon: Home },
    { to: '/ramos', labelKey: 'nav.courses' as const, icon: BookOpen },
    { to: '/tareas', labelKey: 'nav.tasks' as const, icon: ClipboardList },
    {
        to: '/recursos',
        labelKey: 'nav.resources' as const,
        icon: FolderOpen,
    },
    { to: '/notas', labelKey: 'nav.grades' as const, icon: GraduationCap },
    {
        to: '/horario',
        labelKey: 'nav.schedule' as const,
        icon: CalendarDays,
    },
    { to: '/novedades', labelKey: 'nav.news' as const, icon: Newspaper },
    { to: '/buscar', labelKey: 'nav.search' as const, icon: Search },
    { to: '/ajustes', labelKey: 'nav.settings' as const, icon: Settings },
];

interface AppShellProps {
    children: ReactNode;
}

const SOURCE_LABEL = {
    moodle: 'Moodle',
    ellucian: 'Ellucian',
    banner: 'Banner',
} as const;

export function AppShell({ children }: AppShellProps) {
    const { t, i18n } = useTranslation();
    const sync = useSyncController();
    const syncing = sync.state === 'running';
    const locale =
        LANGUAGES.find((language) => language.code === i18n.language)?.locale ??
        'en-US';
    const lastSyncAt = sync.lastRun?.finished_at
        ? new Date(sync.lastRun.finished_at)
        : null;
    const pathname = useRouterState({
        select: (state) => state.location.pathname,
    });

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
                        <SidebarGroupLabel>
                            {t('shell.navigation')}
                        </SidebarGroupLabel>
                        <SidebarGroupContent>
                            <SidebarMenu>
                                {NAV.map((item) => (
                                    <SidebarMenuItem key={item.to}>
                                        <SidebarMenuButton
                                            isActive={pathname === item.to}
                                            tooltip={t(item.labelKey)}
                                            render={<Link to={item.to} />}
                                        >
                                            <item.icon />
                                            <span>{t(item.labelKey)}</span>
                                        </SidebarMenuButton>
                                    </SidebarMenuItem>
                                ))}
                            </SidebarMenu>
                        </SidebarGroupContent>
                    </SidebarGroup>
                </SidebarContent>

                <SidebarFooter>
                    <div className="px-2 py-1 text-xs text-muted-foreground">
                        {t('shell.account')}
                    </div>
                </SidebarFooter>
            </Sidebar>

            <SidebarInset>
                <header className="flex h-14 items-center gap-3 border-b border-border px-4">
                    <SidebarTrigger />
                    <Separator orientation="vertical" className="h-6" />
                    <SyncBadge state={sync.state} />
                    {syncing ? (
                        <div className="flex items-center gap-2">
                            <Progress value={sync.progress} className="w-24" />
                            <span className="text-xs text-muted-foreground">
                                {sync.progress}%
                                {sync.source
                                    ? ` · ${SOURCE_LABEL[sync.source]}`
                                    : ''}
                            </span>
                        </div>
                    ) : null}
                    {lastSyncAt ? (
                        <span className="text-xs text-muted-foreground">
                            {t('shell.lastUpdate', {
                                time: lastSyncAt.toLocaleTimeString(locale, {
                                    hour: '2-digit',
                                    minute: '2-digit',
                                }),
                            })}
                        </span>
                    ) : null}
                    <div className="ml-auto flex items-center gap-2">
                        {syncing ? (
                            <button
                                type="button"
                                className="rounded-md border border-border px-3 py-1 text-sm hover:bg-muted"
                                onClick={sync.cancel}
                            >
                                {t('common.cancel')}
                            </button>
                        ) : null}
                        <button
                            type="button"
                            className="rounded-md border border-border px-3 py-1 text-sm hover:bg-muted disabled:opacity-50"
                            onClick={() => void sync.start()}
                            disabled={syncing}
                        >
                            {syncing ? t('shell.syncing') : t('shell.sync')}
                        </button>
                    </div>
                </header>

                <div className="flex-1 overflow-auto p-6">{children}</div>
            </SidebarInset>
        </SidebarProvider>
    );
}
