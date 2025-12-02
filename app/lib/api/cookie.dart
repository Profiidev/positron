import 'package:cookie_jar/cookie_jar.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';

class SecureCookieStorage implements Storage {
  final FlutterSecureStorage _storage;

  static const String _storageKey = "cookies";

  const SecureCookieStorage(this._storage);

  @override
  Future<void> init(bool persistSession, bool ignoreExpires) async {}

  @override
  Future<void> write(String key, String value) async {
    await _storage.write(key: '$_storageKey-$key', value: value);
  }

  @override
  Future<String?> read(String key) async {
    return _storage.read(key: '$_storageKey-$key');
  }

  @override
  Future<void> delete(String key) async {
    return _storage.delete(key: '$_storageKey-$key');
  }

  @override
  Future<void> deleteAll(List<String> keys) async {
    for (var key in keys) {
      await _storage.delete(key: '$_storageKey-$key');
    }
  }
}
