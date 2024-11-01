let token = $state(localStorage.getItem("token"));

$effect.root(() => {
  if (token) {
    localStorage.setItem("token", token);
  }
});

export const get_token = () => {
  return token;
}

export const set_token = (new_token: string) => {
  token = new_token;
}
