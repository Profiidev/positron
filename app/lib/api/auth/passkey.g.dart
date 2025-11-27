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
);

Map<String, dynamic> _$PublicKeyCredentialRequestToJson(
  PublicKeyCredentialRequest instance,
) => <String, dynamic>{'res': instance.res};

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
