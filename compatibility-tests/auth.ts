/**
 * Authentication endpoint tests for Utopia compatibility verification suite.
 *
 * Tests:
 * - Bootstrap token issuance (POST /api/v1/bootstrap/tokens)
 * - Token revocation (DELETE /api/v1/tokens/:id)
 * - Unauthenticated request rejection (GET /api/v1/accounts without token)
 */

import { http, check, checkErrorEnvelope, checkUnauthorized, BASE_URL, BOOTSTRAP_KEY } from './harness.ts';

export const options = {
  vus: 1,
  iterations: 1,
  thresholds: {
    http_req_duration: ['p(95)<500'],
    // Allow up to 60% failure rate since 3/6 requests intentionally return 401
    http_req_failed: ['rate<0.6'],
  },
};

interface TokenIssuanceResponse {
  data: {
    id: string;
    label: string;
    token: string;
    status: string;
    created_at: string;
  };
}

export default function (): void {
  // -----------------------------------------------------------------------
  // Test 1: Bootstrap token issuance
  // -----------------------------------------------------------------------
  const bootstrapUrl = `${BASE_URL}/api/v1/bootstrap/tokens`;
  const bootstrapPayload = JSON.stringify({ label: 'k6-auth-test' });
  const bootstrapParams = {
    headers: {
      'Content-Type': 'application/json',
      'X-Bootstrap-Key': BOOTSTRAP_KEY,
    },
    tags: { endpoint: 'auth' },
  };

  const bootstrapRes = http.post(bootstrapUrl, bootstrapPayload, bootstrapParams);

  const bootstrapSuccess = check(bootstrapRes, {
    'auth: bootstrap token status is 200': (r) => r.status === 200,
    'auth: bootstrap response has data': (r) => {
      try {
        const body = JSON.parse(r.body as string) as TokenIssuanceResponse;
        return body.data !== undefined;
      } catch {
        return false;
      }
    },
    'auth: token has id (uuid)': (r) => {
      try {
        const body = JSON.parse(r.body as string) as TokenIssuanceResponse;
        return typeof body.data.id === 'string' && body.data.id.length === 36;
      } catch {
        return false;
      }
    },
    'auth: token has label': (r) => {
      try {
        const body = JSON.parse(r.body as string) as TokenIssuanceResponse;
        return typeof body.data.label === 'string' && body.data.label.length > 0;
      } catch {
        return false;
      }
    },
    'auth: token has token string': (r) => {
      try {
        const body = JSON.parse(r.body as string) as TokenIssuanceResponse;
        return typeof body.data.token === 'string' && body.data.token.length > 0;
      } catch {
        return false;
      }
    },
    'auth: token has status': (r) => {
      try {
        const body = JSON.parse(r.body as string) as TokenIssuanceResponse;
        return typeof body.data.status === 'string';
      } catch {
        return false;
      }
    },
    'auth: token has created_at': (r) => {
      try {
        const body = JSON.parse(r.body as string) as TokenIssuanceResponse;
        return typeof body.data.created_at === 'string';
      } catch {
        return false;
      }
    },
  });

  if (!bootstrapSuccess) {
    return; // Cannot proceed without a token
  }

  const tokenBody = JSON.parse(bootstrapRes.body as string) as TokenIssuanceResponse;
  const tokenId = tokenBody.data.id;

  // -----------------------------------------------------------------------
  // Test 2: Authenticated request succeeds with bootstrap token
  // -----------------------------------------------------------------------
  const accountsUrl = `${BASE_URL}/api/v1/accounts`;
  const authHeaders = {
    'Content-Type': 'application/json',
    Authorization: `Bearer ${tokenBody.data.token}`,
  };

  const authedRes = http.get(accountsUrl, {
    headers: authHeaders,
    tags: { endpoint: 'auth' },
  });

  check(authedRes, {
    'auth: authenticated request succeeds (200)': (r) => r.status === 200,
    'auth: response is list envelope': (r) => {
      try {
        const body = JSON.parse(r.body as string);
        return Array.isArray(body.data) && body.meta && body.meta.pagination;
      } catch {
        return false;
      }
    },
  });

  // -----------------------------------------------------------------------
  // Test 3: Unauthenticated request is rejected (401)
  // -----------------------------------------------------------------------
  const unauthRes = http.get(accountsUrl, {
    headers: { 'Content-Type': 'application/json' },
    tags: { endpoint: 'auth' },
  });

  checkUnauthorized(unauthRes, 'auth: unauthenticated request');

  // -----------------------------------------------------------------------
  // Test 4: Token revocation
  // -----------------------------------------------------------------------
  const revokeUrl = `${BASE_URL}/api/v1/tokens/${tokenId}`;
  const revokeRes = http.del(revokeUrl, null, {
    headers: authHeaders,
    tags: { endpoint: 'auth' },
  });

  check(revokeRes, {
    'auth: token revocation returns 204': (r) => r.status === 204,
  });

  // -----------------------------------------------------------------------
  // Test 5: Revoked token is rejected
  // -----------------------------------------------------------------------
  const revokedRes = http.get(accountsUrl, {
    headers: authHeaders,
    tags: { endpoint: 'auth' },
  });

  checkUnauthorized(revokedRes, 'auth: revoked token');

  // -----------------------------------------------------------------------
  // Test 6: Bootstrap with invalid key is rejected
  // -----------------------------------------------------------------------
  const invalidBootstrapRes = http.post(bootstrapUrl, JSON.stringify({ label: 'invalid-key-test' }), {
    headers: {
      'Content-Type': 'application/json',
      'X-Bootstrap-Key': 'invalid-key-12345',
    },
    tags: { endpoint: 'auth' },
  });

  checkErrorEnvelope(invalidBootstrapRes, 'auth: invalid bootstrap key', 401);
}
