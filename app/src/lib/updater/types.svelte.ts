export enum UpdateMessageType {
  AuthStatusUpdated = 'AuthStatusUpdated',
  SetupUpdated = 'SetupUpdated',
  UserInfoUpdated = 'UserInfoUpdated',
  NotesUpdated = 'NotesUpdated',
  TokenInvalid = 'TokenInvalid',
  Disconnected = 'Disconnected',
  Connected = 'Connected',
  CodeExchangeFailed = 'CodeExchangeFailed',
  CodeExchangeMissingCode = 'CodeExchangeMissingCode',
  CodeExchangeMissingVerifier = 'CodeExchangeMissingVerifier',
  AuthSuccess = 'AuthSuccess',
  ConfirmAuth = 'ConfirmAuth',
  ConfirmAuthMissingCode = 'ConfirmAuthMissingCode',
  None = 'None',
  UsersUpdated = 'UsersUpdated',
  AppSettings = 'AppSettings',
  OpenNotes = 'OpenNotes',
  OpenNote = 'OpenNote',
  OpenSettings = 'OpenSettings'
}

// oxlint-disable-next-line consistent-type-definitions
export type UpdateMessage =
  | {
      type:
        | UpdateMessageType.Disconnected
        | UpdateMessageType.TokenInvalid
        | UpdateMessageType.Connected
        | UpdateMessageType.CodeExchangeFailed
        | UpdateMessageType.CodeExchangeMissingCode
        | UpdateMessageType.CodeExchangeMissingVerifier
        | UpdateMessageType.AuthSuccess
        | UpdateMessageType.ConfirmAuthMissingCode
        | UpdateMessageType.AuthStatusUpdated
        | UpdateMessageType.SetupUpdated
        | UpdateMessageType.UserInfoUpdated
        | UpdateMessageType.NotesUpdated
        | UpdateMessageType.UsersUpdated
        | UpdateMessageType.AppSettings
        | UpdateMessageType.OpenNotes
        | UpdateMessageType.OpenSettings;
    }
  | {
      type: UpdateMessageType.ConfirmAuth;
      code: string;
      redirect?: string;
    }
  | {
      type: UpdateMessageType.OpenNote;
      uuid: string;
    };
