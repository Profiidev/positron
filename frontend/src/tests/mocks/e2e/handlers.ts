import { HttpResponse, http, ws } from 'msw';
import * as gen from '$lib/client/msw.gen';
import * as data from './data';
import { type Client, createClient, createConfig } from '$lib/client/client';
import type { ClientOptions } from '$lib/client/types.gen';

const client: Client = createClient(createConfig<ClientOptions>());

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
 * Reuses the generated `handle*` factories (the same mock api the unit
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

  gen.handleIsSetup(({ cookies }) => j(data.isSetupOf(cookies))),
  gen.handleGetOidcSettings(() => j(data.oidcSettings)),
  gen.handleInfo(({ cookies }) =>
    data.isAnonymous(cookies)
      ? (new HttpResponse(null, { status: 401 }) as never)
      : j(data.adminUser)
  ),
  gen.handleAuthConfig(() => j(data.authConfig)),
  gen.handleAccountSettings(() => j(data.accountSettings)),
  gen.handleMailActive(({ cookies }) => j(data.mailActiveOf(cookies))),
  gen.handleGetMailSettings(() => j(data.mailSettings)),
  gen.handleSiteUrl(() => j(data.siteUrl)),
  gen.handleKey(() => j({ key: 'test-public-key' })),

  // Lists (scenario-aware: `mock_scenario=empty` cookie => empty state).
  gen.handleListGroups(({ cookies }) => j(data.groups[scn(cookies)])),
  gen.handleListUsers(({ cookies }) => j(data.users[scn(cookies)])),
  gen.handleListNotes(({ cookies }) =>
    j(data.notes[data.notesScenarioOf(cookies)])
  ),
  gen.handleNotesConfig(({ cookies }) =>
    j(data.notesConfig[data.notesScenarioOf(cookies)])
  ),
  gen.handleListOauthClients(({ cookies }) =>
    j(data.oauthClients[scn(cookies)])
  ),
  gen.handleListOAuthScopes(({ cookies }) => j(data.oauthScopes[scn(cookies)])),
  gen.handleListOAuthPolicies(({ cookies }) =>
    j(data.oauthPolicies[scn(cookies)])
  ),
  gen.handleListPasskeys(({ cookies }) => j(data.passkeys[scn(cookies)])),
  gen.handleListSessions(({ cookies }) => j(data.sessions[scn(cookies)])),
  gen.handleRevokeSession(
    () => new HttpResponse(null, { status: 200 }) as never
  ),
  gen.handleListApod(({ cookies }) => j(data.apodList[scn(cookies)])),
  gen.handleGetApodImageInfo(() => j(data.apodImageInfo)),

  // Simple lists used by detail/create pages.
  gen.handleListGroupsSimple(({ cookies }) =>
    j(data.simpleGroups[scn(cookies)])
  ),
  gen.handleListUsersSimple(({ cookies }) => j(data.simpleUsers[scn(cookies)])),
  gen.handleListUsersNote(({ cookies }) => j(data.noteUsers[scn(cookies)])),
  gen.handleListGroupsOAuthClient(({ cookies }) =>
    j(data.simpleGroups[scn(cookies)])
  ),
  gen.handleListUsersOAuthClient(({ cookies }) =>
    j(data.simpleUsers[scn(cookies)])
  ),
  gen.handleListScopesOAuthClient(({ cookies }) =>
    j(data.simpleScopes[scn(cookies)])
  ),
  gen.handleListPoliciesOAuthScope(({ cookies }) =>
    j(data.simplePolicies[scn(cookies)])
  ),
  gen.handleListGroupsOAuthPolicy(({ cookies }) =>
    j(data.simpleGroups[scn(cookies)])
  ),

  // Details.
  gen.handleGroupInfo(({ params }) =>
    // The uuid is a path param; return a non-admin group for group-staff so its
    // Editable permissions matrix renders (the admin group hides it).
    j(
      params.uuid === 'group-staff' ? data.groupStaffDetails : data.groupDetails
    )
  ),
  gen.handleUserInfo(() => j(data.userDetails)),
  gen.handleInfoOauthClient(() => j(data.oauthClientDetails)),
  gen.handleInfoOAuthScope(() => j(data.oauthScopeDetails)),
  gen.handleInfoOAuthPolicy(() => j(data.oauthPolicyDetails)),
  gen.handleInfoNote(({ cookies }) =>
    j(
      data.isReadonlyNote(cookies) ? data.noteDetailsReadonly : data.noteDetails
    )
  ),
  gen.handleInfoNoteShare(({ cookies }) => j(data.publicNoteOf(cookies))),
  gen.handleListNoteSnapshots(({ cookies }) =>
    j(data.noteSnapshots[scn(cookies)])
  ),
  // `snapshot-missing` reports a 404 so the view page's not-found redirect
  // (back to the note with `?error=not_found`) can be exercised.
  gen.handleInfoNoteSnapshot(({ params }) =>
    params.snapshot_id === 'snapshot-missing'
      ? (new HttpResponse(null, { status: 404 }) as never)
      : j(data.noteSnapshotDetail)
  ),
  // The readonly editor applies an empty yjs update, so serve an empty body.
  gen.handleGetNoteSnapshotContent(
    () => new HttpResponse(new ArrayBuffer(0), { status: 200 }) as never
  ),
  gen.handleDeleteNoteSnapshot(
    () => new HttpResponse(null, { status: 200 }) as never
  ),
  gen.handleRestoreNoteSnapshot(
    () => new HttpResponse(null, { status: 200 }) as never
  ),
  gen.handleShareNotePublic(
    () => new HttpResponse(null, { status: 200 }) as never
  ),
  gen.handleTransferNote(({ cookies }) => {
    if (cookies.mock_scenario === 'transfer-at-limit') {
      return new HttpResponse(null, { status: 409 }) as never;
    }
    return new HttpResponse(null, { status: 200 }) as never;
  }),

  // Mutations return a generic success so submit flows resolve.
  gen.handleCreateGroup(() => j({ uuid: 'group-new' })),
  gen.handleCreateUser(() => j({ uuid: 'user-new' })),
  gen.handleCreateNote(() => j({ id: 'note-new' })),
  gen.handleCreateOauthClient(() =>
    j({ client_id: 'client-new', client_secret: 'secret' })
  ),
  gen.handleCreateOAuthScope(() => j({ uuid: 'scope-new' })),
  gen.handleCreateOAuthPolicy(() => j({ uuid: 'policy-new' })),
  gen.handleSendResetLink(() => new HttpResponse(null, { status: 200 })),

  // Catch-all: any other `/api/*` call resolves with an empty 200 so unmocked
  // Endpoints never crash a page render.
  http.all('*/api/*', () => HttpResponse.json({}))
];

client.setConfig(original);
