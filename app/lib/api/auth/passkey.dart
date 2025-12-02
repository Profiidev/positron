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
