export enum UpdateMessageType {
  AuthStatusUpdated = 'AuthStatusUpdated',
  SetupUpdated = 'SetupUpdated',
  TokenInvalid = 'TokenInvalid',
  Disconnected = 'Disconnected',
  Connected = 'Connected',
  CodeExchangeFailed = 'CodeExchangeFailed',
  CodeExchangeMissingCode = 'CodeExchangeMissingCode',
  CodeExchangeMissingVerifier = 'CodeExchangeMissingVerifier',
  AuthSuccess = 'AuthSuccess',
  ConfirmAuth = 'ConfirmAuth',
  ConfirmAuthMissingCode = 'ConfirmAuthMissingCode'
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
        | UpdateMessageType.SetupUpdated;
    }
  | {
      type: UpdateMessageType.ConfirmAuth;
      code: string;
    };
