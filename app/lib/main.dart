import 'package:app/api/auth/passkey.dart';
import 'package:app/api/auth/password.dart';
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
  int _counter = 0;
  bool obscure = true;
  final formKey = GlobalKey<ShadFormState>();
  final Future<PasswordAuth> auth = PasswordAuth.fromKey();

  void _incrementCounter() async {
    setState(() {
      // This call to setState tells the Flutter framework that something has
      // changed in this State, which causes it to rerun the build method below
      // so that the display can reflect the updated values. If we changed
      // _counter without calling setState(), then the build method would not be
      // called again, and so nothing would appear to happen.
      _counter++;
    });

    final passkeyAuth = PasskeyAuthenticator();

    final request = await startPasskeyAuth();
    var platformRes;
    try {
      platformRes = await passkeyAuth.authenticate(request);
    } on NoCredentialsAvailableException {
      print("No credentials available for this user.");
      return;
    }
    await finishPasskeyAuth(platformRes);
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
      body: Center(
        child: ShadForm(
          key: formKey,
          // Center is a layout widget. It takes a single child and positions it
          // in the middle of the parent.
          child: Column(
            // Column is also a layout widget. It takes a list of children and
            // arranges them vertically. By default, it sizes itself to fit its
            // children horizontally, and tries to be as tall as its parent.
            //
            // Column has various properties to control how it sizes itself and
            // how it positions its children. Here we use mainAxisAlignment to
            // center the children vertically; the main axis here is the vertical
            // axis because Columns are vertical (the cross axis would be
            // horizontal).
            //
            // TRY THIS: Invoke "debug painting" (choose the "Toggle Debug Paint"
            // action in the IDE, or press "p" in the console), to see the
            // wireframe for each widget.
            mainAxisAlignment: MainAxisAlignment.center,
            children: <Widget>[
              const Text('You have pushed the button this many times:'),
              Text(
                '$_counter',
                style: Theme.of(context).textTheme.headlineMedium,
              ),
              ShadInputFormField(
                id: 'username',
                label: const Text('Username'),
                placeholder: const Text('Enter your username'),
                leading: const Padding(
                  padding: EdgeInsets.all(4.0),
                  child: Icon(LucideIcons.user),
                ),
                validator: (v) {
                  if (v.isEmpty) {
                    return 'Please enter your username';
                  }
                  return null;
                },
              ),
              ShadInputFormField(
                id: 'password',
                label: const Text('Password'),
                placeholder: const Text('Enter your password'),
                obscureText: obscure,
                leading: const Padding(
                  padding: EdgeInsets.all(4.0),
                  child: Icon(LucideIcons.lock),
                ),
                trailing: ShadButton(
                  width: 24,
                  height: 24,
                  padding: EdgeInsets.zero,
                  child: Icon(obscure ? LucideIcons.eyeOff : LucideIcons.eye),
                  onPressed: () {
                    setState(() {
                      obscure = !obscure;
                    });
                  },
                ),
                validator: (v) {
                  if (v.isEmpty) {
                    return 'Please enter your password';
                  }
                  return null;
                },
              ),
              ShadButton(
                child: const Text('Shadcn Button'),
                onPressed: () async {
                  if (formKey.currentState!.saveAndValidate()) {
                    print("Form is valid");

                    var formData = formKey.currentState!.value;
                    var username = formData['username'] as String;
                    var password = formData['password'] as String;

                    var passwordAuth = await auth;
                    var res = await passwordAuth.authenticate(
                      username,
                      password,
                    );
                    print("TOTP required: ${res}");
                  } else {
                    print("Form is invalid");
                  }
                },
              ),
            ],
          ),
        ),
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: _incrementCounter,
        tooltip: 'Increment',
        child: const Icon(Icons.add),
      ), // This trailing comma makes auto-formatting nicer for build methods.
    );
  }
}
