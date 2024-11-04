export enum AuthError {
  MissingToken = "MissingToken",
  Other = "Other",
  Passkey = "Passkey",
  Password = "Password",
  Totp = "Totp",
}

export type Passkey = {
  name: string;
  created: string;
  used: string;
};
