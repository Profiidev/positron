export interface NoteActiveEditor {
  clientId: number;
  id?: string;
  name: string;
  color?: string;
}

export interface NoteUserInfo {
  id: string;
  name: string;
}

export interface NoteSnapshotInfo {
  created_at: string;
  id: string;
  note_id: string;
  preview: string;
}

export type NoteShareAccess = 'view' | 'edit';

export interface NoteSharedUserInfo {
  access: NoteShareAccess;
  id: string;
  name: string;
}
