import { HttpResponse, http, ws } from 'msw';
import { client } from '$mocks/msw-runtime';
import * as gen from '$lib/client/msw.gen';
import * as data from './data';

/**
 * No-op WebSocket mock for the updater channel. The app opens this socket on
 * every page (see `connectWebsocket`); without a handler the preview server
 * answers with `404`, which clutters the test output. Accept the connection and
 * do nothing (never forward to a real server) so no update events fire.
 */
const updaterWs = ws.link('*/api/ws/updater');

/**
 * No-op mock for the public-note update channel. The public-share page opens
 * this socket to learn when the owner revokes access; accept it and stay quiet
 * so the page renders without a dangling connection error.
 */
const publicUpdaterWs = ws.link('*/api/notes/update/*');

/**
 * App-login device channel. The login page opens this socket and renders a QR
 * code from the first message it receives, so emit a fake device code on
 * connection to drive the "App Login" flow.
 */
const appLoginWs = ws.link('*/api/auth/app/device_login');

/**
 * Reuses the generated `*MswHandler` factories (the same mock api the unit
 * tests use). The factories build their URL from the client's `baseUrl`; in the
 * preview server every `/api/*` request is host-rewritten to the backend by
 * `handleFetch`, so we build the handlers with `baseUrl = '*'` to match any
 * origin, then restore the real config for the SDK's actual requests.
 */
const original = client.getConfig();
client.setConfig({ ...original, baseUrl: '*' });

const j = (d: unknown) => HttpResponse.json(d as never) as never;
const scn = (cookies: Record<string, string>) => data.scenarioOf(cookies);

export const handlers = [
  updaterWs.addEventListener('connection', () => {}),
  publicUpdaterWs.addEventListener('connection', () => {}),
  // oxlint-disable-next-line no-shadow
  appLoginWs.addEventListener('connection', ({ client }) => {
    client.send('device-login-code');
  }),

  gen.isSetupMswHandler(({ cookies }) => j(data.isSetupOf(cookies))),
  gen.getOidcSettingsMswHandler(() => j(data.oidcSettings)),
  gen.infoMswHandler(({ cookies }) =>
    data.isAnonymous(cookies)
      ? (new HttpResponse(null, { status: 401 }) as never)
      : j(data.adminUser)
  ),
  gen.authConfigMswHandler(() => j(data.authConfig)),
  gen.accountSettingsMswHandler(() => j(data.accountSettings)),
  gen.mailActiveMswHandler(({ cookies }) => j(data.mailActiveOf(cookies))),
  gen.getMailSettingsMswHandler(() => j(data.mailSettings)),
  gen.siteUrlMswHandler(() => j(data.siteUrl)),
  gen.keyMswHandler(() => j({ key: 'test-public-key' })),

  // Lists (scenario-aware: `mock_scenario=empty` cookie => empty state).
  gen.listGroupsMswHandler(({ cookies }) => j(data.groups[scn(cookies)])),
  gen.listUsersMswHandler(({ cookies }) => j(data.users[scn(cookies)])),
  gen.listNotesMswHandler(({ cookies }) =>
    j(data.notes[data.notesScenarioOf(cookies)])
  ),
  gen.notesConfigMswHandler(({ cookies }) =>
    j(data.notesConfig[data.notesScenarioOf(cookies)])
  ),
  gen.listOauthClientsMswHandler(({ cookies }) =>
    j(data.oauthClients[scn(cookies)])
  ),
  gen.listOAuthScopesMswHandler(({ cookies }) =>
    j(data.oauthScopes[scn(cookies)])
  ),
  gen.listOAuthPoliciesMswHandler(({ cookies }) =>
    j(data.oauthPolicies[scn(cookies)])
  ),
  gen.listPasskeysMswHandler(({ cookies }) => j(data.passkeys[scn(cookies)])),
  gen.listSessionsMswHandler(({ cookies }) => j(data.sessions[scn(cookies)])),
  gen.revokeSessionMswHandler(
    () => new HttpResponse(null, { status: 200 }) as never
  ),
  gen.listApodMswHandler(({ cookies }) => j(data.apodList[scn(cookies)])),
  gen.getApodImageInfoMswHandler(() => j(data.apodImageInfo)),

  // Simple lists used by detail/create pages.
  gen.listGroupsSimpleMswHandler(({ cookies }) =>
    j(data.simpleGroups[scn(cookies)])
  ),
  gen.listUsersSimpleMswHandler(({ cookies }) =>
    j(data.simpleUsers[scn(cookies)])
  ),
  gen.listUsersNoteMswHandler(({ cookies }) => j(data.noteUsers[scn(cookies)])),
  gen.listGroupsOAuthClientMswHandler(({ cookies }) =>
    j(data.simpleGroups[scn(cookies)])
  ),
  gen.listUsersOAuthClientMswHandler(({ cookies }) =>
    j(data.simpleUsers[scn(cookies)])
  ),
  gen.listScopesOAuthClientMswHandler(({ cookies }) =>
    j(data.simpleScopes[scn(cookies)])
  ),
  gen.listPoliciesOAuthScopeMswHandler(({ cookies }) =>
    j(data.simplePolicies[scn(cookies)])
  ),
  gen.listGroupsOAuthPolicyMswHandler(({ cookies }) =>
    j(data.simpleGroups[scn(cookies)])
  ),

  // Details.
  gen.groupInfoMswHandler(({ params }) =>
    // The uuid is a path param; return a non-admin group for group-staff so its
    // Editable permissions matrix renders (the admin group hides it).
    j(
      params.uuid === 'group-staff' ? data.groupStaffDetails : data.groupDetails
    )
  ),
  gen.userInfoMswHandler(() => j(data.userDetails)),
  gen.infoOauthClientMswHandler(() => j(data.oauthClientDetails)),
  gen.infoOAuthScopeMswHandler(() => j(data.oauthScopeDetails)),
  gen.infoOAuthPolicyMswHandler(() => j(data.oauthPolicyDetails)),
  gen.infoNoteMswHandler(({ cookies }) =>
    j(
      data.isReadonlyNote(cookies) ? data.noteDetailsReadonly : data.noteDetails
    )
  ),
  gen.infoNoteShareMswHandler(({ cookies }) => j(data.publicNoteOf(cookies))),
  gen.listNoteSnapshotsMswHandler(({ cookies }) =>
    j(data.noteSnapshots[scn(cookies)])
  ),
  // `snapshot-missing` reports a 404 so the view page's not-found redirect
  // (back to the note with `?error=not_found`) can be exercised.
  gen.infoNoteSnapshotMswHandler(({ params }) =>
    params.snapshot_id === 'snapshot-missing'
      ? (new HttpResponse(null, { status: 404 }) as never)
      : j(data.noteSnapshotDetail)
  ),
  // The readonly editor applies an empty yjs update, so serve an empty body.
  gen.getNoteSnapshotContentMswHandler(
    () => new HttpResponse(new ArrayBuffer(0), { status: 200 }) as never
  ),
  gen.deleteNoteSnapshotMswHandler(
    () => new HttpResponse(null, { status: 200 }) as never
  ),
  gen.restoreNoteSnapshotMswHandler(
    () => new HttpResponse(null, { status: 200 }) as never
  ),
  gen.shareNotePublicMswHandler(
    () => new HttpResponse(null, { status: 200 }) as never
  ),
  gen.transferNoteMswHandler(({ cookies }) => {
    if (cookies.mock_scenario === 'transfer-at-limit') {
      return new HttpResponse(null, { status: 409 }) as never;
    }
    return new HttpResponse(null, { status: 200 }) as never;
  }),

  // Mutations return a generic success so submit flows resolve.
  gen.createGroupMswHandler(() => j({ uuid: 'group-new' })),
  gen.createUserMswHandler(() => j({ uuid: 'user-new' })),
  gen.createNoteMswHandler(() => j({ id: 'note-new' })),
  gen.createOauthClientMswHandler(() =>
    j({ client_id: 'client-new', client_secret: 'secret' })
  ),
  gen.createOAuthScopeMswHandler(() => j({ uuid: 'scope-new' })),
  gen.createOAuthPolicyMswHandler(() => j({ uuid: 'policy-new' })),
  gen.sendResetLinkMswHandler(() => new HttpResponse(null, { status: 200 })),

  // Catch-all: any other `/api/*` call resolves with an empty 200 so unmocked
  // Endpoints never crash a page render.
  http.all('*/api/*', () => HttpResponse.json({}))
];

client.setConfig(original);
