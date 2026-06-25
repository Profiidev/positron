// oxlint-disable no-null no-underscore-dangle no-void no-plusplus prefer-spread prefer-exponentiation-operator
import { Channel, invoke } from '@tauri-apps/api/core';
import * as decoding from 'lib0/decoding';
import * as encoding from 'lib0/encoding';
import { ObservableV2 } from 'lib0/observable';
import * as awarenessProtocol from 'y-protocols/awareness';
import * as syncProtocol from 'y-protocols/sync';
import type * as Y from 'yjs';
import type {
  CreateNoteResult,
  NoteInfo,
  NoteShareAccess,
  NoteSnapshotDetail,
  NoteSnapshotInfo,
  NotesConfig,
  ShareEntry,
  SimpleUserInfo,
  TransferNoteResult
} from './notes-types';

export type * from './notes-types';

export const listNotesStore = async (): Promise<NoteInfo[] | undefined> => {
  try {
    return await invoke<NoteInfo[]>('list_notes_store');
  } catch {
    return undefined;
  }
};

export const listNotes = async (): Promise<NoteInfo[] | undefined> => {
  try {
    return await invoke<NoteInfo[]>('list_notes');
  } catch {
    return undefined;
  }
};

export const noteInfoStore = async (
  id: string
): Promise<NoteInfo | undefined> => {
  try {
    return await invoke<NoteInfo | undefined>('get_note_store', { id });
  } catch {
    return undefined;
  }
};

export const noteInfo = async (uuid: string): Promise<NoteInfo | undefined> => {
  try {
    return await invoke<NoteInfo>('note_info', { uuid });
  } catch {
    return undefined;
  }
};

export const notesConfig = async (): Promise<NotesConfig | undefined> => {
  try {
    return await invoke<NotesConfig>('notes_config');
  } catch {
    return undefined;
  }
};

export const listUsersNote = async (): Promise<SimpleUserInfo[]> => {
  try {
    return await invoke<SimpleUserInfo[]>('list_users_note');
  } catch {
    return [];
  }
};

export const listNoteSnapshots = async (
  noteUuid: string
): Promise<NoteSnapshotInfo[]> => {
  try {
    return await invoke<NoteSnapshotInfo[]>('list_note_snapshots', {
      noteUuid
    });
  } catch {
    return [];
  }
};

export const noteSnapshotInfo = async (
  snapshotId: string
): Promise<NoteSnapshotDetail | undefined> => {
  try {
    return await invoke<NoteSnapshotDetail>('note_snapshot_info', {
      snapshotId
    });
  } catch {
    return undefined;
  }
};

export const noteSnapshotContent = async (
  snapshotId: string
): Promise<Uint8Array | undefined> => {
  try {
    const res = await invoke<number[]>('note_snapshot_content', { snapshotId });
    return new Uint8Array(res);
  } catch {
    return undefined;
  }
};

export const noteContent = async (
  noteId: string
): Promise<Uint8Array | undefined> => {
  try {
    const res = await invoke<number[]>('note_content', { id: noteId });
    return new Uint8Array(res);
  } catch {
    return undefined;
  }
};

export const saveNoteContent = async (
  noteId: string,
  content: Uint8Array
): Promise<boolean> => {
  try {
    await invoke('save_note_content', {
      content: Array.from(content),
      id: noteId
    });
    return true;
  } catch {
    return false;
  }
};

export const editNote = async (
  noteId: string,
  title: string
): Promise<boolean> => {
  try {
    await invoke('edit_note', { noteId, title });
    return true;
  } catch {
    return false;
  }
};

export const shareNote = async (
  noteId: string,
  sharedWith: ShareEntry[]
): Promise<boolean> => {
  try {
    await invoke('share_note', { noteId, sharedWith });
    return true;
  } catch {
    return false;
  }
};

export const shareNotePublic = async (
  noteId: string,
  publicAccess: NoteShareAccess | null
): Promise<boolean> => {
  try {
    await invoke('share_note_public', { noteId, publicAccess });
    return true;
  } catch {
    return false;
  }
};

export const restoreNoteSnapshot = async (
  snapshotId: string
): Promise<boolean> => {
  try {
    await invoke('restore_note_snapshot', { snapshotId });
    return true;
  } catch {
    return false;
  }
};

export const deleteNote = async (noteId: string): Promise<boolean> => {
  try {
    await invoke('delete_note', { noteId });
    return true;
  } catch {
    return false;
  }
};

export const deleteNoteSnapshot = async (
  snapshotId: string
): Promise<boolean> => {
  try {
    await invoke('delete_note_snapshot', { snapshotId });
    return true;
  } catch {
    return false;
  }
};

export const createNote = async (title: string): Promise<CreateNoteResult> => {
  try {
    const note = await invoke<{ id: string }>('create_note', { title });
    return { id: note.id, ok: true };
  } catch (error) {
    return { error: error === 'limit' ? 'limit' : 'other', ok: false };
  }
};

export const transferNote = async (
  noteId: string,
  newOwnerId: string
): Promise<TransferNoteResult> => {
  try {
    await invoke('transfer_note', { newOwnerId, noteId });
    return { ok: true };
  } catch (error) {
    return { error: error === 'limit' ? 'limit' : 'other', ok: false };
  }
};

type WebsocketMessage =
  | {
      type: 'Data';
      data: Uint8Array;
    }
  | {
      type: 'Close';
    };

class NoteConnection {
  private readonly note_id: string;
  private readonly channel_id: string;

  private constructor(note_id: string, channel_id: string) {
    this.note_id = note_id;
    this.channel_id = channel_id;
  }

  public static async connect(
    note_id: string,
    listener: (data: WebsocketMessage) => void
  ): Promise<NoteConnection> {
    const receive = new Channel<WebsocketMessage>();
    // oxlint-disable-next-line prefer-add-event-listener
    receive.onmessage = listener;

    try {
      const uuid = await invoke<string>('connect_note', {
        channel: receive,
        note: note_id
      });

      return new NoteConnection(note_id, uuid);
    } catch (error) {
      // oxlint-disable-next-line no-console
      console.error(error);
      throw error;
    }
  }

  public async send(data: Uint8Array): Promise<void> {
    await invoke('send_note', {
      data,
      uuid: this.channel_id
    });
  }

  public async disconnect(): Promise<void> {
    await invoke('disconnect_note', {
      uuid: this.channel_id
    });
  }
}

const messageSync = 0;
const messageAwareness = 1;
const messageQueryAwareness = 3;

type MessageHandler = (
  encoder: encoding.Encoder,
  decoder: decoding.Decoder,
  provider: TauriWebsocketProvider,
  emitSynced: boolean
) => void;

interface TauriWebsocketProviderEvents {
  'connection-close': (provider: TauriWebsocketProvider) => void;
  status: (event: {
    status: 'connected' | 'disconnected' | 'connecting';
  }) => void;
  'connection-error': (
    error: unknown,
    provider: TauriWebsocketProvider
  ) => void;
  sync: (state: boolean) => void;
}

interface TauriWebsocketProviderOptions {
  /** Whether to connect immediately on construction. */
  connect?: boolean;
  /** Provide a shared awareness instance. */
  awareness?: awarenessProtocol.Awareness;
  /**
   * Maximum delay (ms) between reconnect attempts. Reconnects use exponential
   * backoff starting at 100ms, capped at this value.
   */
  maxBackoffTime?: number;
  /**
   * Accepted for API compatibility with `y-websocket`. The Tauri provider never
   * uses cross-tab BroadcastChannel, so this is effectively always `true`.
   */
  disableBc?: boolean;
}

/**
 * Yjs provider that syncs a `Y.Doc` over a Tauri {@link NoteConnection} instead
 * of a raw WebSocket. Ported from `y-websocket`, stripped of the BroadcastChannel,
 * URL/param/protocol/polyfill handling and socket-timeout logic. Reconnect with
 * exponential backoff is driven by the connection's `Close` event.
 */
export class TauriWebsocketProvider extends ObservableV2<TauriWebsocketProviderEvents> {
  public readonly noteId: string;
  public readonly doc: Y.Doc;
  public readonly awareness: awarenessProtocol.Awareness;

  private readonly messageHandlers: MessageHandler[] = [];

  private conn: NoteConnection | null = null;
  private connecting = false;
  private connected = false;
  private shouldConnect: boolean;
  private _synced = false;
  /** Replies generated before the connection promise resolves are buffered here. */
  private pendingSends: Uint8Array[] = [];

  private readonly maxBackoffTime: number;
  private unsuccessfulReconnects = 0;
  private reconnectTimeout: ReturnType<typeof setTimeout> | null = null;

  private readonly _updateHandler: (
    update: Uint8Array,
    origin: unknown
  ) => void;
  private readonly _awarenessUpdateHandler: (changes: {
    added: number[];
    updated: number[];
    removed: number[];
  }) => void;

  public constructor(
    noteId: string,
    doc: Y.Doc,
    {
      connect = true,
      awareness = new awarenessProtocol.Awareness(doc),
      maxBackoffTime = 2500
    }: TauriWebsocketProviderOptions = {}
  ) {
    super();
    this.noteId = noteId;
    this.doc = doc;
    this.awareness = awareness;
    this.shouldConnect = connect;
    this.maxBackoffTime = maxBackoffTime;

    this.messageHandlers[messageSync] = (
      encoder,
      decoder,
      provider,
      emitSynced
    ) => {
      encoding.writeVarUint(encoder, messageSync);
      const syncMessageType = syncProtocol.readSyncMessage(
        decoder,
        encoder,
        provider.doc,
        provider
      );
      if (
        emitSynced &&
        syncMessageType === syncProtocol.messageYjsSyncStep2 &&
        !provider.synced
      ) {
        provider.synced = true;
      }
    };
    this.messageHandlers[messageQueryAwareness] = (
      encoder,
      _decoder,
      provider
    ) => {
      encoding.writeVarUint(encoder, messageAwareness);
      encoding.writeVarUint8Array(
        encoder,
        awarenessProtocol.encodeAwarenessUpdate(
          provider.awareness,
          Array.from(provider.awareness.getStates().keys())
        )
      );
    };
    this.messageHandlers[messageAwareness] = (_encoder, decoder, provider) => {
      awarenessProtocol.applyAwarenessUpdate(
        provider.awareness,
        decoding.readVarUint8Array(decoder),
        provider
      );
    };

    // Send local document updates to the remote peer.
    this._updateHandler = (update, origin) => {
      if (origin !== this) {
        const encoder = encoding.createEncoder();
        encoding.writeVarUint(encoder, messageSync);
        syncProtocol.writeUpdate(encoder, update);
        this.broadcast(encoding.toUint8Array(encoder));
      }
    };
    this.doc.on('update', this._updateHandler);

    // Send local awareness changes to the remote peer.
    this._awarenessUpdateHandler = ({ added, updated, removed }) => {
      const changedClients = added.concat(updated, removed);
      const encoder = encoding.createEncoder();
      encoding.writeVarUint(encoder, messageAwareness);
      encoding.writeVarUint8Array(
        encoder,
        awarenessProtocol.encodeAwarenessUpdate(this.awareness, changedClients)
      );
      this.broadcast(encoding.toUint8Array(encoder));
    };
    this.awareness.on('update', this._awarenessUpdateHandler);

    if (connect) {
      this.connect();
    }
  }

  public get synced(): boolean {
    return this._synced;
  }

  public set synced(state: boolean) {
    if (this._synced !== state) {
      this._synced = state;
      // @ts-expect-error idk y-websocket does the same
      this.emit('synced', [state]);
      this.emit('sync', [state]);
    }
  }

  public connect(): void {
    this.shouldConnect = true;
    if (!this.connected && this.conn === null && !this.connecting) {
      this.setupConn();
    }
  }

  public disconnect(): void {
    this.shouldConnect = false;
    if (this.reconnectTimeout !== null) {
      clearTimeout(this.reconnectTimeout);
      this.reconnectTimeout = null;
    }
    this.closeConn();
  }

  public destroy(): void {
    this.disconnect();
    this.awareness.off('update', this._awarenessUpdateHandler);
    this.doc.off('update', this._updateHandler);
    super.destroy();
  }

  private setupConn(): void {
    if (!this.shouldConnect || this.conn !== null || this.connecting) {
      return;
    }
    this.connecting = true;
    this.connected = false;
    this.synced = false;
    this.emit('status', [{ status: 'connecting' }]);

    // Identifies this connection attempt so a stale `Close` cannot tear down a
    // Newer connection.
    let conn: NoteConnection | null = null;

    NoteConnection.connect(this.noteId, (message) => {
      if (conn !== null && this.conn !== conn) {
        // Event from a connection we already replaced; ignore.
        return;
      }
      if (message.type === 'Close') {
        this.closeConn();
        return;
      }
      const encoder = this.readMessage(new Uint8Array(message.data), true);
      if (encoding.length(encoder) > 1) {
        this.send(encoding.toUint8Array(encoder));
      }
    })
      .then((c) => {
        conn = c;
        // `disconnect` may have been called while connecting.
        if (!this.shouldConnect) {
          this.connecting = false;
          void c.disconnect();
          return;
        }
        this.conn = c;
        this.connecting = false;
        this.connected = true;
        this.unsuccessfulReconnects = 0;
        this.emit('status', [{ status: 'connected' }]);

        // Flush replies that arrived before the connection resolved.
        for (const data of this.pendingSends) {
          void c.send(data);
        }
        this.pendingSends = [];

        // Always send sync step 1 when connected.
        const encoder = encoding.createEncoder();
        encoding.writeVarUint(encoder, messageSync);
        syncProtocol.writeSyncStep1(encoder, this.doc);
        void conn.send(encoding.toUint8Array(encoder));

        // Broadcast local awareness state.
        if (this.awareness.getLocalState() !== null) {
          const awarenessEncoder = encoding.createEncoder();
          encoding.writeVarUint(awarenessEncoder, messageAwareness);
          encoding.writeVarUint8Array(
            awarenessEncoder,
            awarenessProtocol.encodeAwarenessUpdate(this.awareness, [
              this.doc.clientID
            ])
          );
          void c.send(encoding.toUint8Array(awarenessEncoder));
        }
      })
      .catch((error: unknown) => {
        this.connecting = false;
        this.unsuccessfulReconnects++;
        this.emit('connection-error', [error, this]);
        this.scheduleReconnect();
      });
  }

  /**
   * Tear down the active connection and schedule a reconnect (a no-op while
   * `shouldConnect` is false). Driven by the connection's `Close` event.
   */
  private closeConn(): void {
    // oxlint-disable-next-line prefer-destructuring
    const conn = this.conn;
    if (conn !== null) {
      this.conn = null;
      void conn.disconnect();
    }
    this.connecting = false;
    this.pendingSends = [];
    if (this.connected) {
      this.connected = false;
      this.synced = false;
      // All remote clients are gone once disconnected.
      awarenessProtocol.removeAwarenessStates(
        this.awareness,
        Array.from(this.awareness.getStates().keys()).filter(
          (client) => client !== this.doc.clientID
        ),
        this
      );
      this.emit('status', [{ status: 'disconnected' }]);
      this.emit('connection-close', [this]);
    } else {
      this.unsuccessfulReconnects++;
    }
    this.scheduleReconnect();
  }

  /** Reconnect with exponential backoff starting at 100ms. */
  private scheduleReconnect(): void {
    if (!this.shouldConnect || this.reconnectTimeout !== null) {
      return;
    }
    const delay = Math.min(
      Math.pow(2, this.unsuccessfulReconnects) * 100,
      this.maxBackoffTime
    );
    this.reconnectTimeout = setTimeout(() => {
      this.reconnectTimeout = null;
      this.setupConn();
    }, delay);
  }

  private readMessage(buf: Uint8Array, emitSynced: boolean): encoding.Encoder {
    const decoder = decoding.createDecoder(buf);
    const encoder = encoding.createEncoder();
    while (decoding.hasContent(decoder)) {
      const messageType = decoding.readVarUint(decoder);
      const messageHandler = this.messageHandlers[messageType];
      if (messageHandler) {
        messageHandler(encoder, decoder, this, emitSynced);
      } else {
        // oxlint-disable-next-line no-console
        console.error('Unable to compute message');
      }
    }
    return encoder;
  }

  /** Send during the initial handshake, buffering until the connection is ready. */
  private send(data: Uint8Array): void {
    if (this.conn) {
      void this.conn.send(data);
    } else {
      this.pendingSends.push(data);
    }
  }

  /** Send an ongoing update; dropped while disconnected (peer resyncs on reconnect). */
  private broadcast(data: Uint8Array): void {
    if (this.connected && this.conn) {
      void this.conn.send(data);
    }
  }
}
