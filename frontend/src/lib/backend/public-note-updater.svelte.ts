import { invalidate } from '$app/navigation';
import { createWebsocket } from '@profidev/pleiades/backend';

const websocket = createWebsocket<any>();

export const connectPublicNoteUpdater = (noteId: string) =>
  websocket.connect(() => {
    invalidate(`/api/notes/management/${noteId}/public`).catch(() => {});
  }, `/api/notes/update/${noteId}`);

export const disconnectPublicNoteUpdater = () => {
  websocket.disconnect();
};
