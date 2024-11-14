export enum RequestError {
  Other,
  Unauthorized,
  Conflict,
}

export enum ResponseType {
  Json = "Json",
  Text = "Text",
  None = "None",
}

export enum ContentType {
  Json = "application/json",
  UrlFrom = "x-www-form-urlencoded",
  Bytes = "application/octet-stream",
}
