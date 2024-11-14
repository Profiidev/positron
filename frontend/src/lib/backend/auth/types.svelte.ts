export interface Passkey {
  name: string;
  created: string;
  used: string;
}

export interface TotpCode {
  qr: string;
  code: string;
}

export interface OAuthParams {
  code: string;
  name: string;
}
