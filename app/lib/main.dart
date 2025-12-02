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

final _rootNavigatorKey = GlobalKey<NavigatorState>();
final _shellNavigatorHomeKey = GlobalKey<NavigatorState>();
final _shellNavigatorProfileKey = GlobalKey<NavigatorState>();
final _shellNavigatorLoginKey = GlobalKey<NavigatorState>();

final GoRouter _router = GoRouter(
  initialLocation: "/",
  navigatorKey: _rootNavigatorKey,
  debugLogDiagnostics: true,
  routes: [
    StatefulShellRoute.indexedStack(
      builder: (context, state, navigationShell) {
        return ScaffoldWithNestedNavigation(navigationShell: navigationShell);
      },
      branches: [
        StatefulShellBranch(
          navigatorKey: _shellNavigatorHomeKey,
          routes: [
            GoRoute(
              path: "/",
              name: "home",
              builder: (context, state) {
                return const Text("Home Page");
              },
            ),
          ],
        ),
        StatefulShellBranch(
          navigatorKey: _shellNavigatorLoginKey,
          routes: [
            GoRoute(
              path: "/login",
              name: "login",
              builder: (context, state) => const LoginPage(),
            ),
          ],
        ),
        StatefulShellBranch(
          navigatorKey: _shellNavigatorProfileKey,
          routes: [
            GoRoute(
              path: "/profile",
              name: "profile",
              builder: (context, state) => const Text("Profile Page"),
            ),
          ],
        ),
      ],
    ),
  ],
);

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
      routerConfig: _router,
    );
  }
}

class ScaffoldWithNestedNavigation extends StatelessWidget {
  final formKey = GlobalKey<ShadFormState>();
  final StatefulNavigationShell navigationShell;

  ScaffoldWithNestedNavigation({super.key, required this.navigationShell});

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
    await api.finishPasskeyAuth(platformRes, request.id);
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      bottomNavigationBar: NavigationBar(
        onDestinationSelected: (idx) {
          navigationShell.goBranch(idx);
        },
        selectedIndex: navigationShell.currentIndex,
        destinations: [
          NavigationDestination(icon: Icon(Icons.home), label: 'Home'),
          NavigationDestination(icon: Icon(Icons.login), label: 'Login'),
          NavigationDestination(icon: Icon(Icons.person), label: 'Profile'),
        ],
      ),
      body: navigationShell,
      floatingActionButton: FloatingActionButton(
        onPressed: _incrementCounter,
        tooltip: 'Increment',
        child: const Icon(Icons.add),
      ), // This trailing comma makes auto-formatting nicer for build methods.
    );
  }
}
