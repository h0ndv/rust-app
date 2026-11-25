import React from 'react';
import ReactDOM from 'react-dom/client';

import App from './App';
import { ToastProvider } from './components/ui/toast';
import i18n, { resolveLanguage } from './i18n';
import { api } from './lib/api';
import { initTheme } from './lib/theme';
import './styles.css';

async function bootstrap(): Promise<void> {
    let language = 'en';
    let theme = 'system';

    try {
        const settings = await api.getSettings();
        language = resolveLanguage(settings.language);
        theme = settings.theme;
    } catch (error) {
        console.warn('No se pudo leer el idioma guardado', error);
    }

    initTheme(theme);
    await i18n.changeLanguage(language);
    document.documentElement.lang = language;

    ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
        <React.StrictMode>
            <ToastProvider>
                <App />
            </ToastProvider>
        </React.StrictMode>,
    );
}

void bootstrap();
