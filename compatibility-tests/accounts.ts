/**
 * Accounts endpoint tests for Utopia compatibility verification suite.
 *
 * Tests:
 * - List accounts (GET /api/v1/accounts)
 * - Get account (GET /api/v1/accounts/:id)
 * - Create account (POST /api/v1/accounts)
 * - Update account (PUT /api/v1/accounts/:id)
 * - Delete account (DELETE /api/v1/accounts/:id)
 *
 * Validates responses against Firefly-III format using the shared harness.
 */

import {
  http,
  check,
  authenticatedHeaders,
  checkListEnvelope,
  checkSingleEnvelope,
  checkResourceStructure,
  checkNoContent,
  checkErrorEnvelope,
  checkPaginationConsistency,
  BASE_URL,
  type FireflyResource,
  type FireflyListEnvelope,
  type FireflySingleEnvelope,
} from './harness.ts';

export const options = {
  vus: 1,
  iterations: 1,
  thresholds: {
    http_req_duration: ['p(95)<500'],
    // Allow up to 20% failure rate since 1 request intentionally returns 404 (verify deletion)
    http_req_failed: ['rate<0.2'],
  },
};

type AccountResource = FireflyResource;
type ListEnvelope = FireflyListEnvelope<AccountResource>;
type SingleEnvelope = FireflySingleEnvelope<AccountResource>;

export default function (): void {
  const headers = authenticatedHeaders();
  const tags = { endpoint: 'accounts' };

  // -----------------------------------------------------------------------
  // Test 1: List accounts
  // -----------------------------------------------------------------------
  const listRes = http.get(`${BASE_URL}/api/v1/accounts`, { headers, tags });

  checkListEnvelope(listRes, 'accounts: list', 1);
  checkPaginationConsistency(listRes, 'accounts: list');

  // Validate each resource in the list
  const listBody = JSON.parse(listRes.body as string) as ListEnvelope;
  if (listBody.data.length > 0) {
    checkResourceStructure(listBody.data[0], 'accounts: list resource');
  }

  // Check account type field values
  check(listRes, {
    "accounts: list resources have type 'accounts'": (r) => {
      try {
        const body = JSON.parse(r.body as string) as ListEnvelope;
        return body.data.every((item) => item.type === 'accounts');
      } catch {
        return false;
      }
    },
    'accounts: list resources have required attributes': (r) => {
      try {
        const body = JSON.parse(r.body as string) as ListEnvelope;
        return body.data.every(
          (item) =>
            typeof item.attributes.name === 'string' &&
            typeof item.attributes.type === 'string' &&
            typeof item.attributes.currency_code === 'string' &&
            typeof item.attributes.current_balance === 'string' &&
            typeof item.attributes.active === 'boolean',
        );
      } catch {
        return false;
      }
    },
  });

  // Get first account ID for subsequent tests
  const firstAccountId = listBody.data[0]?.id;
  if (!firstAccountId) {
    return; // Cannot proceed without an account
  }

  // -----------------------------------------------------------------------
  // Test 2: Get single account
  // -----------------------------------------------------------------------
  const getRes = http.get(`${BASE_URL}/api/v1/accounts/${firstAccountId}`, {
    headers,
    tags,
  });

  checkSingleEnvelope(getRes, 'accounts: get');
  checkResourceStructure(JSON.parse(getRes.body as string).data, 'accounts: get resource');

  check(getRes, {
    'accounts: get returns correct id': (r) => {
      try {
        const body = JSON.parse(r.body as string) as SingleEnvelope;
        return body.data.id === firstAccountId;
      } catch {
        return false;
      }
    },
    'accounts: get has currency_decimal_places': (r) => {
      try {
        const body = JSON.parse(r.body as string) as SingleEnvelope;
        return typeof body.data.attributes.currency_decimal_places === 'number';
      } catch {
        return false;
      }
    },
    'accounts: get has primary currency fields': (r) => {
      try {
        const body = JSON.parse(r.body as string) as SingleEnvelope;
        return (
          typeof body.data.attributes.primary_currency_code === 'string' &&
          typeof body.data.attributes.primary_currency_name === 'string' &&
          typeof body.data.attributes.primary_currency_symbol === 'string' &&
          typeof body.data.attributes.primary_currency_decimal_places === 'number'
        );
      } catch {
        return false;
      }
    },
  });

  // -----------------------------------------------------------------------
  // Test 3: Create account
  // -----------------------------------------------------------------------
  const createPayload = JSON.stringify({
    name: 'k6 Test Account',
    type: 'asset',
    currency_code: 'JPY',
    active: true,
    include_net_worth: true,
  });

  const createRes = http.post(`${BASE_URL}/api/v1/accounts`, createPayload, {
    headers,
    tags,
  });

  check(createRes, {
    'accounts: create returns 201': (r) => r.status === 201,
  });

  checkSingleEnvelope(createRes, 'accounts: create');

  const createBody = JSON.parse(createRes.body as string) as SingleEnvelope;
  const newAccountId = createBody.data.id;

  check(createRes, {
    'accounts: create returns correct name': (r) => {
      try {
        const body = JSON.parse(r.body as string) as SingleEnvelope;
        return body.data.attributes.name === 'k6 Test Account';
      } catch {
        return false;
      }
    },
    'accounts: create returns correct type': (r) => {
      try {
        const body = JSON.parse(r.body as string) as SingleEnvelope;
        return body.data.attributes.type === 'asset';
      } catch {
        return false;
      }
    },
    'accounts: create returns correct currency': (r) => {
      try {
        const body = JSON.parse(r.body as string) as SingleEnvelope;
        return body.data.attributes.currency_code === 'JPY';
      } catch {
        return false;
      }
    },
  });

  // -----------------------------------------------------------------------
  // Test 4: Update account
  // -----------------------------------------------------------------------
  const updatePayload = JSON.stringify({
    name: 'k6 Updated Account',
    active: false,
  });

  const updateRes = http.put(`${BASE_URL}/api/v1/accounts/${newAccountId}`, updatePayload, { headers, tags });

  check(updateRes, {
    'accounts: update returns 200': (r) => r.status === 200,
  });

  checkSingleEnvelope(updateRes, 'accounts: update');

  check(updateRes, {
    'accounts: update returns correct name': (r) => {
      try {
        const body = JSON.parse(r.body as string) as SingleEnvelope;
        return body.data.attributes.name === 'k6 Updated Account';
      } catch {
        return false;
      }
    },
    'accounts: update returns correct active status': (r) => {
      try {
        const body = JSON.parse(r.body as string) as SingleEnvelope;
        return body.data.attributes.active === false;
      } catch {
        return false;
      }
    },
  });

  // -----------------------------------------------------------------------
  // Test 5: Delete account
  // -----------------------------------------------------------------------
  const deleteRes = http.del(`${BASE_URL}/api/v1/accounts/${newAccountId}`, null, {
    headers,
    tags,
  });

  checkNoContent(deleteRes, 'accounts: delete');

  // Verify deletion: GET should return 404
  const getDeletedRes = http.get(`${BASE_URL}/api/v1/accounts/${newAccountId}`, {
    headers,
    tags,
  });

  checkErrorEnvelope(getDeletedRes, 'accounts: get deleted', 404);

  // -----------------------------------------------------------------------
  // Test 6: List with pagination
  // -----------------------------------------------------------------------
  const paginatedRes = http.get(`${BASE_URL}/api/v1/accounts?page=1&limit=2`, {
    headers,
    tags,
  });

  checkListEnvelope(paginatedRes, 'accounts: list paginated', 0);
  checkPaginationConsistency(paginatedRes, 'accounts: list paginated');

  check(paginatedRes, {
    'accounts: paginated list respects limit': (r) => {
      try {
        const body = JSON.parse(r.body as string) as ListEnvelope;
        return body.data.length <= 2;
      } catch {
        return false;
      }
    },
    'accounts: paginated list has correct per_page': (r) => {
      try {
        const body = JSON.parse(r.body as string) as ListEnvelope;
        return body.meta.pagination.per_page === 2;
      } catch {
        return false;
      }
    },
  });

  // -----------------------------------------------------------------------
  // Test 7: List with type filter
  // -----------------------------------------------------------------------
  const filteredRes = http.get(`${BASE_URL}/api/v1/accounts?type=asset`, {
    headers,
    tags,
  });

  checkListEnvelope(filteredRes, 'accounts: list filtered', 0);

  check(filteredRes, {
    'accounts: filtered list returns only matching type': (r) => {
      try {
        const body = JSON.parse(r.body as string) as ListEnvelope;
        return body.data.every((item) => item.attributes.type === 'asset');
      } catch {
        return false;
      }
    },
  });
}
