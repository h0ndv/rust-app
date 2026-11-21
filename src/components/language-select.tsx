import { LANGUAGES, type Language } from '@/i18n';

interface LanguageSelectProps {
    value: string;
    onChange: (language: Language) => void;
    id?: string;
    className?: string;
    ariaLabel?: string;
}

export function LanguageSelect({
    value,
    onChange,
    id,
    className,
    ariaLabel,
}: LanguageSelectProps) {
    return (
        <select
            id={id}
            className={className}
            aria-label={ariaLabel}
            value={value}
            onChange={(event) => onChange(event.target.value as Language)}
        >
            {LANGUAGES.map((language) => (
                <option key={language.code} value={language.code}>
                    {language.label}
                </option>
            ))}
        </select>
    );
}
