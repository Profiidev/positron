export enum AuthError {
  MissingToken = "MissingToken",
  Other = "Other",
  Passkey = "Passkey",
  Password = "Password",
  Totp = "Totp",
  Conflict = "Conflict",
}

export interface Passkey {
  name: string;
  created: string;
  used: string;
}

export interface TotpInfo {
  enabled: boolean;
  created?: string;
  last_used?: string;
}

export interface TotpCode {
  qr: string;
  code: string;
}

export interface PasswordInfo {
  last_login: string;
  last_special_access: string;
}

export interface OAuthParams {
  code: string;
}
