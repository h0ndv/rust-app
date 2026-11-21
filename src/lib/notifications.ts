import {
    isPermissionGranted,
    requestPermission,
} from '@tauri-apps/plugin-notification';

export async function ensureNotificationPermission(): Promise<boolean> {
    if (await isPermissionGranted()) {
        return true;
    }
    return (await requestPermission()) === 'granted';
}
