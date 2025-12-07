// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'password.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

KeyResponse _$KeyResponseFromJson(Map<String, dynamic> json) =>
    KeyResponse(key: json['key'] as String);

Map<String, dynamic> _$KeyResponseToJson(KeyResponse instance) =>
    <String, dynamic>{'key': instance.key};

PasswordAuthRequest _$PasswordAuthRequestFromJson(Map<String, dynamic> json) =>
    PasswordAuthRequest(
      email: json['email'] as String,
      password: json['password'] as String,
    );

Map<String, dynamic> _$PasswordAuthRequestToJson(
  PasswordAuthRequest instance,
) => <String, dynamic>{'email': instance.email, 'password': instance.password};

PasswordAuthResponse _$PasswordAuthResponseFromJson(
  Map<String, dynamic> json,
) => PasswordAuthResponse(totp: json['totp'] as bool);

Map<String, dynamic> _$PasswordAuthResponseToJson(
  PasswordAuthResponse instance,
) => <String, dynamic>{'totp': instance.totp};

TotpConfirmRequest _$TotpConfirmRequestFromJson(Map<String, dynamic> json) =>
    TotpConfirmRequest(code: json['code'] as String);

Map<String, dynamic> _$TotpConfirmRequestToJson(TotpConfirmRequest instance) =>
    <String, dynamic>{'code': instance.code};

Passkey _$PasskeyFromJson(Map<String, dynamic> json) => Passkey(
  name: json['name'] as String,
  created: json['created'] as String,
  used: json['used'] as String,
);

Map<String, dynamic> _$PasskeyToJson(Passkey instance) => <String, dynamic>{
  'name': instance.name,
  'created': instance.created,
  'used': instance.used,
};
