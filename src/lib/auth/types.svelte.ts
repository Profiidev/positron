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
};

export interface TotpInfo {
  enabled: boolean;
  created: string | undefined;
  last_used: string | undefined;
}

export interface TotpCode {
  qr: string;
  code: string;
}

