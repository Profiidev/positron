import 'package:app/api/client.dart';
import 'package:app/pages/login.dart';
import 'package:flutter/material.dart';
import 'package:passkeys/authenticator.dart';
import 'package:passkeys/types.dart';
import 'package:shadcn_ui/shadcn_ui.dart';
import 'package:go_router/go_router.dart';

void main() {
  runApp(const Positron());
}

class Positron extends StatelessWidget {
  const Positron({super.key});

  @override
  Widget build(BuildContext context) {
    return ShadApp.router(
      title: 'Flutter Demo',
      theme: ShadThemeData(
        brightness: Brightness.light,
        colorScheme: const ShadBlueColorScheme.light(),
      ),
      darkTheme: ShadThemeData(
        brightness: Brightness.dark,
        colorScheme: const ShadBlueColorScheme.dark(),
      ),
      routerConfig: GoRouter(
        routes: [
          GoRoute(
            path: "/",
            builder: (context, state) =>
                const MyHomePage(title: 'Flutter Demo Home Page'),
          ),
        ],
      ),
    );
  }
}

class MyHomePage extends StatefulWidget {
  const MyHomePage({super.key, required this.title});

  // This widget is the home page of your application. It is stateful, meaning
  // that it has a State object (defined below) that contains fields that affect
  // how it looks.

  // This class is the configuration for the state. It holds the values (in this
  // case the title) provided by the parent (in this case the App widget) and
  // used by the build method of the State. Fields in a Widget subclass are
  // always marked "final".

  final String title;

  @override
  State<MyHomePage> createState() => _MyHomePageState();
}

class _MyHomePageState extends State<MyHomePage> {
  final formKey = GlobalKey<ShadFormState>();

  void _incrementCounter() async {
    final passkeyAuth = PasskeyAuthenticator();
    final api = DioClient.create();

    final request = await api.startPasskeyAuth();
    var platformRes;
    try {
      platformRes = await passkeyAuth.authenticate(request.res.publicKey);
    } on NoCredentialsAvailableException {
      print("No credentials available for this user.");
      return;
    }
    await api.finishPasskeyAuth(platformRes);
  }

  @override
  Widget build(BuildContext context) {
    // This method is rerun every time setState is called, for instance as done
    // by the _incrementCounter method above.
    //
    // The Flutter framework has been optimized to make rerunning build methods
    // fast, so that you can just rebuild anything that needs updating rather
    // than having to individually change instances of widgets.
    return Scaffold(
      bottomNavigationBar: NavigationBar(
        destinations: [
          NavigationDestination(icon: Icon(Icons.home), label: 'Home'),
          NavigationDestination(icon: Icon(Icons.business), label: 'Business'),
          NavigationDestination(icon: Icon(Icons.school), label: 'School'),
        ],
      ),
      body: LoginPage(),
      floatingActionButton: FloatingActionButton(
        onPressed: _incrementCounter,
        tooltip: 'Increment',
        child: const Icon(Icons.add),
      ), // This trailing comma makes auto-formatting nicer for build methods.
    );
  }
}
