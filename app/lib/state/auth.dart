import 'package:app/api/cookie.dart';
import 'package:app/api/rest_api.dart';
import 'package:cookie_jar/cookie_jar.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';

Future<bool> _checkLoggedIn() async {
  const secureStorage = FlutterSecureStorage();
  const cookieStorage = SecureCookieStorage(secureStorage);
  final cookieJar = PersistCookieJar(
    storage: cookieStorage,
    ignoreExpires: false,
  );

  final cookies = cookieJar.loadForRequest(Uri.parse(baseUrl));
  bool loggedIn = false;

  for (var cookie in await cookies) {
    if (cookie.name == "token" && cookie.value.isNotEmpty) {
      loggedIn = true;
      break;
    }
  }

  return loggedIn;
}

abstract class LoggedInEvent {}

class LoggedInChecked extends LoggedInEvent {}

class LoggedInCubit extends Cubit<bool> {
  LoggedInCubit() : super(true);

  void checkLoggedIn() async => emit(await _checkLoggedIn());
}

class LoggedInStateWidget extends StatelessWidget {
  const LoggedInStateWidget({super.key, required this.builder});

  final BlocWidgetBuilder<bool> builder;

  @override
  Widget build(BuildContext context) {
    return BlocProvider(
      create: (_) => LoggedInCubit()..checkLoggedIn(),
      child: BlocBuilder<LoggedInCubit, bool>(builder: builder),
    );
  }
}
