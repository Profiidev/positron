export enum TokenType {
  Auth = "Auth",
  SpecialAccess = "SpecialAccess",
  TotpRequired = "TotpRequired",
}

type Claims = {
  exp: number;
  iss: string;
  sub: string;
  type: TokenType;
};

let auth = $state(localStorage.getItem("token"));
let other = $state(undefined as string | undefined);

$effect.root(() => {
  $effect(() => {
    if (auth) {
      localStorage.setItem("token", auth);
    } else {
      localStorage.removeItem("token");
    }
  });
});

export const get_token = (type: TokenType) => {
  if (auth && type === TokenType.Auth && get_valid_for(auth) > 0) {
    return auth;
  } else {
    if (other && get_token_type(other) === type && get_valid_for(other) > 0) {
      return other;
    }
  }
};

export const set_token = (token: string, type: TokenType) => {
  if (type === TokenType.Auth) {
    auth = token;
  } else {
    other = token;
  }
};

export const get_token_type = (token: string) => {
  return get_claims(token).type;
};

export const clear_tokens = () => {
  auth = null;
  other = undefined;
};

export const get_uuid = () => {
  let token = get_token(TokenType.Auth);
  if (token) {
    return get_claims(token).sub;
  }
};

const get_claims = (token: string) => {
  let claims_part = token.split(".")[1];
  return JSON.parse(atob(claims_part)) as Claims;
};

const get_valid_for = (token: string) => {
  return get_valid_for_claims(get_claims(token));
};

const get_valid_for_claims = (token: Claims) => {
  let timestamp = Math.floor(new Date().getTime() / 1000);
  return token.exp - timestamp;
};
