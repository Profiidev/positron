import 'dart:convert';

import 'package:http/http.dart';
import 'package:json_annotation/json_annotation.dart';
import 'package:passkeys/types.dart';

part 'passkey.g.dart';

var root_uri = "profidev.io";

@JsonSerializable()
class PublicKeyCredentialRequest {
  final PublicKeyCredentialRequestOptions res;

  PublicKeyCredentialRequest(this.res);

  factory PublicKeyCredentialRequest.fromJson(Map<String, dynamic> json) =>
      _$PublicKeyCredentialRequestFromJson(json);
  Map<String, dynamic> toJson() => _$PublicKeyCredentialRequestToJson(this);
}

@JsonSerializable()
class PublicKeyCredentialRequestOptions {
  final AuthenticateRequestType publicKey;

  PublicKeyCredentialRequestOptions({required this.publicKey});

  factory PublicKeyCredentialRequestOptions.fromJson(
    Map<String, dynamic> json,
  ) => _$PublicKeyCredentialRequestOptionsFromJson(json);
  Map<String, dynamic> toJson() =>
      _$PublicKeyCredentialRequestOptionsToJson(this);
}

Future<AuthenticateRequestType> startPasskeyAuth() async {
  var url = Uri.https(root_uri, '/backend/auth/passkey/start_authentication');
  var response = await get(url);

  if (response.statusCode != 200) {
    throw Exception('Failed to start passkey authentication');
  }

  var body =
      jsonDecode(utf8.decode(response.bodyBytes)) as Map<String, dynamic>;
  print(body);
  var options = PublicKeyCredentialRequest.fromJson(body).res.publicKey;

  return options;
}

Future<void> finishPasskeyAuth(AuthenticateResponseType res) async {
  var url = Uri.https(root_uri, '/backend/auth/passkey/finish_authentication');

  var response = await post(
    url,
    headers: {'Content-Type': 'application/json'},
    body: jsonEncode(res.toJson()),
  );

  if (response.statusCode != 200) {
    throw Exception('Failed to finish passkey authentication');
  }
}
