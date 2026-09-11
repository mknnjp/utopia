/**
 * Transactions endpoint tests for Utopia compatibility verification suite.
 *
 * Tests:
 * - List transactions (GET /api/v1/transactions)
 * - Get transaction (GET /api/v1/transactions/:id)
 * - Create transaction (POST /api/v1/transactions)
 * - Update transaction (PUT /api/v1/transactions/:id)
 * - Delete transaction (DELETE /api/v1/transactions/:id)
 * - List account transactions (GET /api/v1/accounts/:id/transactions)
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

type TransactionResource = FireflyResource;
type AccountResource = FireflyResource;
type ListEnvelope<T> = FireflyListEnvelope<T>;
type SingleEnvelope<T> = FireflySingleEnvelope<T>;

export default function (): void {
  const headers = authenticatedHeaders();
  const tags = { endpoint: 'transactions' };

  // Get an account ID for transaction tests
  const accountsRes = http.get(`${BASE_URL}/api/v1/accounts?limit=1`, {
    headers,
    tags,
  });
  const accountsBody = JSON.parse(accountsRes.body as string) as ListEnvelope<AccountResource>;
  const accountId = accountsBody.data[0]?.id;

  if (!accountId) {
    console.warn('No accounts available for transaction tests');
    return;
  }

  // -----------------------------------------------------------------------
  // Test 1: List transactions
  // -----------------------------------------------------------------------
  const listRes = http.get(`${BASE_URL}/api/v1/transactions`, { headers, tags });

  checkListEnvelope(listRes, 'transactions: list', 1);
  checkPaginationConsistency(listRes, 'transactions: list');

  const listBody = JSON.parse(listRes.body as string) as ListEnvelope<TransactionResource>;

  if (listBody.data.length > 0) {
    checkResourceStructure(listBody.data[0], 'transactions: list resource');
  }

  check(listRes, {
    "transactions: list resources have type 'transactions'": (r) => {
      try {
        const body = JSON.parse(r.body as string) as ListEnvelope<TransactionResource>;
        return body.data.every((item) => item.type === 'transactions');
      } catch {
        return false;
      }
    },
    'transactions: list resources have required attributes': (r) => {
      try {
        const body = JSON.parse(r.body as string) as ListEnvelope<TransactionResource>;
        return body.data.every(
          (item) =>
            typeof item.attributes.description === 'string' &&
            typeof item.attributes.amount === 'string' &&
            typeof item.attributes.currency_code === 'string' &&
            typeof item.attributes.type === 'string' &&
            typeof item.attributes.reconciled === 'boolean',
        );
      } catch {
        return false;
      }
    },
  });

  // Get first transaction ID for subsequent tests
  const firstTxId = listBody.data[0]?.id;
  if (!firstTxId) {
    return;
  }

  // -----------------------------------------------------------------------
  // Test 2: Get single transaction
  // -----------------------------------------------------------------------
  const getRes = http.get(`${BASE_URL}/api/v1/transactions/${firstTxId}`, {
    headers,
    tags,
  });

  checkSingleEnvelope(getRes, 'transactions: get');
  checkResourceStructure(JSON.parse(getRes.body as string).data, 'transactions: get resource');

  check(getRes, {
    'transactions: get returns correct id': (r) => {
      try {
        const body = JSON.parse(r.body as string) as SingleEnvelope<TransactionResource>;
        return body.data.id === firstTxId;
      } catch {
        return false;
      }
    },
    'transactions: get has group_id': (r) => {
      try {
        const body = JSON.parse(r.body as string) as SingleEnvelope<TransactionResource>;
        return typeof body.data.attributes.group_id === 'string';
      } catch {
        return false;
      }
    },
    'transactions: get has user field': (r) => {
      try {
        const body = JSON.parse(r.body as string) as SingleEnvelope<TransactionResource>;
        return typeof body.data.attributes.user === 'string';
      } catch {
        return false;
      }
    },
    'transactions: get has date field': (r) => {
      try {
        const body = JSON.parse(r.body as string) as SingleEnvelope<TransactionResource>;
        return typeof body.data.attributes.date === 'string';
      } catch {
        return false;
      }
    },
  });

  // -----------------------------------------------------------------------
  // Test 3: Create withdrawal transaction
  // -----------------------------------------------------------------------
  const groupId = 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
    const r = (Math.random() * 16) | 0;
    const v = c === 'x' ? r : (r & 0x3) | 0x8;
    return v.toString(16);
  });
  const createPayload = JSON.stringify({
    group_id: groupId,
    transaction_type: 'withdrawal',
    description: 'k6 test withdrawal',
    amount: '1500.00',
    currency_code: 'JPY',
    date: '2026-06-27T10:00:00Z',
    source_id: accountId,
    category_name: 'Testing',
    notes: 'Created by k6 compatibility test',
    reconciled: false,
  });

  const createRes = http.post(`${BASE_URL}/api/v1/transactions`, createPayload, {
    headers,
    tags,
  });

  check(createRes, {
    'transactions: create returns 201': (r) => r.status === 201,
  });

  checkSingleEnvelope(createRes, 'transactions: create');

  const createBody = JSON.parse(createRes.body as string) as SingleEnvelope<TransactionResource>;
  const newTxId = createBody.data.id;

  check(createRes, {
    'transactions: create returns correct description': (r) => {
      try {
        const body = JSON.parse(r.body as string) as SingleEnvelope<TransactionResource>;
        return body.data.attributes.description === 'k6 test withdrawal';
      } catch {
        return false;
      }
    },
    'transactions: create returns correct amount': (r) => {
      try {
        const body = JSON.parse(r.body as string) as SingleEnvelope<TransactionResource>;
        return body.data.attributes.amount === '1500.00';
      } catch {
        return false;
      }
    },
    'transactions: create returns correct type': (r) => {
      try {
        const body = JSON.parse(r.body as string) as SingleEnvelope<TransactionResource>;
        return body.data.attributes.type === 'withdrawal';
      } catch {
        return false;
      }
    },
    'transactions: create returns correct group_id': (r) => {
      try {
        const body = JSON.parse(r.body as string) as SingleEnvelope<TransactionResource>;
        return body.data.attributes.group_id === groupId;
      } catch {
        return false;
      }
    },
  });

  // -----------------------------------------------------------------------
  // Test 4: Update transaction
  // -----------------------------------------------------------------------
  const updatePayload = JSON.stringify({
    description: 'k6 updated withdrawal',
    amount: '2000.00',
    reconciled: true,
  });

  const updateRes = http.put(`${BASE_URL}/api/v1/transactions/${newTxId}`, updatePayload, { headers, tags });

  check(updateRes, {
    'transactions: update returns 200': (r) => r.status === 200,
  });

  checkSingleEnvelope(updateRes, 'transactions: update');

  check(updateRes, {
    'transactions: update returns correct description': (r) => {
      try {
        const body = JSON.parse(r.body as string) as SingleEnvelope<TransactionResource>;
        return body.data.attributes.description === 'k6 updated withdrawal';
      } catch {
        return false;
      }
    },
    'transactions: update returns correct amount': (r) => {
      try {
        const body = JSON.parse(r.body as string) as SingleEnvelope<TransactionResource>;
        return body.data.attributes.amount === '2000.00';
      } catch {
        return false;
      }
    },
    'transactions: update returns correct reconciled': (r) => {
      try {
        const body = JSON.parse(r.body as string) as SingleEnvelope<TransactionResource>;
        return body.data.attributes.reconciled === true;
      } catch {
        return false;
      }
    },
  });

  // -----------------------------------------------------------------------
  // Test 5: Delete transaction
  // -----------------------------------------------------------------------
  const deleteRes = http.del(`${BASE_URL}/api/v1/transactions/${newTxId}`, null, {
    headers,
    tags,
  });

  checkNoContent(deleteRes, 'transactions: delete');

  // Verify deletion: GET should return 404
  const getDeletedRes = http.get(`${BASE_URL}/api/v1/transactions/${newTxId}`, {
    headers,
    tags,
  });

  checkErrorEnvelope(getDeletedRes, 'transactions: get deleted', 404);

  // -----------------------------------------------------------------------
  // Test 6: List account transactions
  // -----------------------------------------------------------------------
  const accountTxRes = http.get(`${BASE_URL}/api/v1/accounts/${accountId}/transactions`, { headers, tags });

  checkListEnvelope(accountTxRes, 'transactions: list by account', 0);
  checkPaginationConsistency(accountTxRes, 'transactions: list by account');

  // -----------------------------------------------------------------------
  // Test 7: List with pagination
  // -----------------------------------------------------------------------
  const paginatedRes = http.get(`${BASE_URL}/api/v1/transactions?page=1&limit=2`, { headers, tags });

  checkListEnvelope(paginatedRes, 'transactions: list paginated', 0);
  checkPaginationConsistency(paginatedRes, 'transactions: list paginated');

  check(paginatedRes, {
    'transactions: paginated list respects limit': (r) => {
      try {
        const body = JSON.parse(r.body as string) as ListEnvelope<TransactionResource>;
        return body.data.length <= 2;
      } catch {
        return false;
      }
    },
  });

  // -----------------------------------------------------------------------
  // Test 8: List with type filter
  // -----------------------------------------------------------------------
  const filteredRes = http.get(`${BASE_URL}/api/v1/transactions?type=withdrawal`, { headers, tags });

  checkListEnvelope(filteredRes, 'transactions: list filtered', 0);

  check(filteredRes, {
    'transactions: filtered list returns only matching type': (r) => {
      try {
        const body = JSON.parse(r.body as string) as ListEnvelope<TransactionResource>;
        return body.data.every((item) => item.attributes.type === 'withdrawal');
      } catch {
        return false;
      }
    },
  });
}
