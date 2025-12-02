import 'package:json_annotation/json_annotation.dart';
import 'package:passkeys/types.dart';

part 'passkey.g.dart';

@JsonSerializable()
class PublicKeyCredentialRequest {
  final PublicKeyCredentialRequestOptions res;
  final String id;

  PublicKeyCredentialRequest(this.res, this.id);

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

@JsonSerializable()
class PublicKeyCredentialCreationOptions {
  final RegisterRequestType publicKey;

  PublicKeyCredentialCreationOptions({required this.publicKey});

  factory PublicKeyCredentialCreationOptions.fromJson(
    Map<String, dynamic> json,
  ) => _$PublicKeyCredentialCreationOptionsFromJson(json);
  Map<String, dynamic> toJson() =>
      _$PublicKeyCredentialCreationOptionsToJson(this);
}

@JsonSerializable()
class PublicKeyCredentialCreationResponse {
  final RegisterResponseType reg;
  final String name;

  PublicKeyCredentialCreationResponse({required this.reg, required this.name});

  factory PublicKeyCredentialCreationResponse.fromJson(
    Map<String, dynamic> json,
  ) => _$PublicKeyCredentialCreationResponseFromJson(json);
  Map<String, dynamic> toJson() =>
      _$PublicKeyCredentialCreationResponseToJson(this);
}
