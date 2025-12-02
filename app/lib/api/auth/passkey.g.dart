// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'passkey.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

PublicKeyCredentialRequest _$PublicKeyCredentialRequestFromJson(
  Map<String, dynamic> json,
) => PublicKeyCredentialRequest(
  PublicKeyCredentialRequestOptions.fromJson(
    json['res'] as Map<String, dynamic>,
  ),
  json['id'] as String,
);

Map<String, dynamic> _$PublicKeyCredentialRequestToJson(
  PublicKeyCredentialRequest instance,
) => <String, dynamic>{'res': instance.res, 'id': instance.id};

PublicKeyCredentialRequestOptions _$PublicKeyCredentialRequestOptionsFromJson(
  Map<String, dynamic> json,
) => PublicKeyCredentialRequestOptions(
  publicKey: AuthenticateRequestType.fromJson(
    json['publicKey'] as Map<String, dynamic>,
  ),
);

Map<String, dynamic> _$PublicKeyCredentialRequestOptionsToJson(
  PublicKeyCredentialRequestOptions instance,
) => <String, dynamic>{'publicKey': instance.publicKey};

PublicKeyCredentialCreationOptions _$PublicKeyCredentialCreationOptionsFromJson(
  Map<String, dynamic> json,
) => PublicKeyCredentialCreationOptions(
  publicKey: RegisterRequestType.fromJson(
    json['publicKey'] as Map<String, dynamic>,
  ),
);

Map<String, dynamic> _$PublicKeyCredentialCreationOptionsToJson(
  PublicKeyCredentialCreationOptions instance,
) => <String, dynamic>{'publicKey': instance.publicKey};

PublicKeyCredentialCreationResponse
_$PublicKeyCredentialCreationResponseFromJson(Map<String, dynamic> json) =>
    PublicKeyCredentialCreationResponse(
      reg: RegisterResponseType.fromJson(json['reg'] as Map<String, dynamic>),
      name: json['name'] as String,
    );

Map<String, dynamic> _$PublicKeyCredentialCreationResponseToJson(
  PublicKeyCredentialCreationResponse instance,
) => <String, dynamic>{'reg': instance.reg, 'name': instance.name};
