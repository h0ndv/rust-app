import React from 'react';
import ReactDOM from 'react-dom/client';

import App from './App';
import i18n, { resolveLanguage } from './i18n';
import { api } from './lib/api';
import './styles.css';

async function bootstrap(): Promise<void> {
    let language = 'en';

    try {
        const settings = await api.getSettings();
        language = resolveLanguage(settings.language);
    } catch (error) {
        console.warn('No se pudo leer el idioma guardado', error);
    }

    await i18n.changeLanguage(language);
    document.documentElement.lang = language;

    ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
        <React.StrictMode>
            <App />
        </React.StrictMode>,
    );
}

void bootstrap();
