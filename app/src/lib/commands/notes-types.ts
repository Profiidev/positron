export type NoteShareAccess = 'view' | 'edit';

export interface SimpleUserInfo {
  id: string;
  name: string;
}

export interface SharedUserInfo {
  id: string;
  name: string;
  access: NoteShareAccess;
}

export interface NoteInfo {
  can_edit: boolean;
  id: string;
  is_owner: boolean;
  owner: SimpleUserInfo;
  preview: string;
  public_access?: NoteShareAccess | null;
  shared_with: SharedUserInfo[];
  title: string;
}

export interface NoteSnapshotInfo {
  created_at: string;
  id: string;
  note_id: string;
  preview: string;
}

export interface NoteSnapshotDetail {
  created_at: string;
  note_id: string;
  title: string;
}

export interface NotesConfig {
  max_per_user?: number;
}

export interface ShareEntry {
  user_id: string;
  access: NoteShareAccess;
}

export type CreateNoteResult =
  | { ok: true; id: string }
  | { ok: false; error: 'limit' | 'other' };

export type TransferNoteResult =
  | { ok: true }
  | { ok: false; error: 'limit' | 'other' };
