import 'package:app/api/cookie.dart';
import 'package:app/api/rest_api.dart';
import 'package:cookie_jar/cookie_jar.dart';
import 'package:dio/dio.dart';
import 'package:dio_cookie_manager/dio_cookie_manager.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';

class DioClient {
  static RestClient create() {
    const secureStorage = FlutterSecureStorage();
    const cookieStorage = SecureCookieStorage(secureStorage);
    final cookieJar = PersistCookieJar(
      storage: cookieStorage,
      ignoreExpires: false,
    );

    final dio = Dio();

    dio.interceptors.addAll([
      LogInterceptor(
        request: true,
        requestBody: true,
        responseBody: true,
        error: true,
      ),
      CookieManager(cookieJar),
    ]);

    return RestClient(dio);
  }
}
