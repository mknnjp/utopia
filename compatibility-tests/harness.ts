/**
 * Shared test harness for Utopia compatibility verification suite.
 *
 * Provides:
 * - Base URL configuration from APP_BASE_URL environment variable
 * - Authentication token retrieval (bootstrap token issuance)
 * - Response validation helpers for Firefly-III format
 * - Dynamic field exclusion for strict mode comparison
 */

import { check, sleep } from 'k6';
import http from 'k6/http';

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

const BASE_URL = __ENV.APP_BASE_URL || 'http://localhost:80';
const BOOTSTRAP_KEY = __ENV.BOOTSTRAP_KEY || 'replace-me-with-long-random-bootstrap-secret';

// ---------------------------------------------------------------------------
// Dynamic fields excluded from strict comparison
// ---------------------------------------------------------------------------

const DYNAMIC_FIELDS = ['created_at', 'updated_at', 'id', 'request_id', 'current_balance_date', 'token'];

// ---------------------------------------------------------------------------
// Authentication
// ---------------------------------------------------------------------------

let cachedToken: string | null = null;

/**
 * Obtain a bootstrap token for authenticated API calls.
 * Caches the token for reuse across test iterations.
 */
export function getAuthToken(): string {
  if (cachedToken) {
    return cachedToken;
  }

  const url = `${BASE_URL}/api/v1/bootstrap/tokens`;
  const payload = JSON.stringify({ label: 'k6-compat-test' });
  const params = {
    headers: {
      'Content-Type': 'application/json',
      'X-Bootstrap-Key': BOOTSTRAP_KEY,
    },
  };

  const res = http.post(url, payload, params);

  const success = check(res, {
    'bootstrap token issued': (r) => r.status === 200,
    'token response has data': (r) => {
      try {
        const body = JSON.parse(r.body as string);
        return body.data && typeof body.data.token === 'string';
      } catch {
        return false;
      }
    },
  });

  if (!success) {
    throw new Error(`Failed to obtain bootstrap token: status=${res.status}, body=${res.body}`);
  }

  const body = JSON.parse(res.body as string);
  cachedToken = body.data.token;
  return cachedToken!;
}

// ---------------------------------------------------------------------------
// HTTP helpers
// ---------------------------------------------------------------------------

export interface RequestParams {
  headers?: Record<string, string>;
  tags?: Record<string, string>;
}

export function authenticatedHeaders(extra?: Record<string, string>): Record<string, string> {
  const token = getAuthToken();
  return {
    'Content-Type': 'application/json',
    Authorization: `Bearer ${token}`,
    ...extra,
  };
}

// ---------------------------------------------------------------------------
// Response validation helpers
// ---------------------------------------------------------------------------

/**
 * Check that a response has the Firefly-III list envelope structure:
 * { "data": [...], "meta": { "pagination": {...} } }
 */
export function checkListEnvelope(res: http.Response, endpoint: string, expectedMinItems = 0): boolean {
  const checks: Record<string, (r: typeof res) => boolean> = {
    [`${endpoint}: status 200`]: (r) => r.status === 200,
    [`${endpoint}: has data array`]: (r) => {
      try {
        const body = JSON.parse(r.body as string);
        return Array.isArray(body.data);
      } catch {
        return false;
      }
    },
    [`${endpoint}: has meta.pagination`]: (r) => {
      try {
        const body = JSON.parse(r.body as string);
        return (
          body.meta &&
          body.meta.pagination &&
          typeof body.meta.pagination.total === 'number' &&
          typeof body.meta.pagination.count === 'number' &&
          typeof body.meta.pagination.per_page === 'number' &&
          typeof body.meta.pagination.current_page === 'number' &&
          typeof body.meta.pagination.total_pages === 'number'
        );
      } catch {
        return false;
      }
    },
    [`${endpoint}: data count >= ${expectedMinItems}`]: (r) => {
      try {
        const body = JSON.parse(r.body as string);
        return body.data.length >= expectedMinItems;
      } catch {
        return false;
      }
    },
  };

  return check(res, checks);
}

/**
 * Check that a response has the Firefly-III single envelope structure:
 * { "data": {...} }
 */
export function checkSingleEnvelope(res: http.Response, endpoint: string): boolean {
  const checks: Record<string, (r: typeof res) => boolean> = {
    [`${endpoint}: status 200`]: (r) => r.status === 200,
    [`${endpoint}: has data object`]: (r) => {
      try {
        const body = JSON.parse(r.body as string);
        return body.data && typeof body.data === 'object';
      } catch {
        return false;
      }
    },
    [`${endpoint}: data has type`]: (r) => {
      try {
        const body = JSON.parse(r.body as string);
        return typeof body.data.type === 'string';
      } catch {
        return false;
      }
    },
    [`${endpoint}: data has id`]: (r) => {
      try {
        const body = JSON.parse(r.body as string);
        return typeof body.data.id === 'string';
      } catch {
        return false;
      }
    },
    [`${endpoint}: data has attributes`]: (r) => {
      try {
        const body = JSON.parse(r.body as string);
        return body.data.attributes && typeof body.data.attributes === 'object';
      } catch {
        return false;
      }
    },
    [`${endpoint}: data has links array`]: (r) => {
      try {
        const body = JSON.parse(r.body as string);
        return Array.isArray(body.data.links);
      } catch {
        return false;
      }
    },
  };

  return check(res, checks);
}

/**
 * Firefly-III compatible resource structure interface.
 */
export interface FireflyResource {
  type: string;
  id: string;
  attributes: Record<string, unknown>;
  links: unknown[];
}

/**
 * Firefly-III list envelope interface.
 */
export interface FireflyListEnvelope<T> {
  data: T[];
  meta: {
    pagination: {
      total: number;
      count: number;
      per_page: number;
      current_page: number;
      total_pages: number;
    };
  };
}

/**
 * Firefly-III single envelope interface.
 */
export interface FireflySingleEnvelope<T> {
  data: T;
}

/**
 * Check that a resource object has the required Firefly-III fields:
 * type, id, attributes, links
 */
export function checkResourceStructure(resource: FireflyResource, endpoint: string): boolean {
  return check(resource, {
    [`${endpoint}: resource has type`]: (r) => typeof r.type === 'string',
    [`${endpoint}: resource has id`]: (r) => typeof r.id === 'string',
    [`${endpoint}: resource has attributes`]: (r) => r.attributes && typeof r.attributes === 'object',
    [`${endpoint}: resource has links`]: (r) => Array.isArray(r.links),
  });
}

/**
 * Check that a response is a 204 No Content (for DELETE operations).
 */
export function checkNoContent(res: http.Response, endpoint: string): boolean {
  return check(res, {
    [`${endpoint}: status 204`]: (r) => r.status === 204,
  });
}

/**
 * Check that a response is a Firefly-III error envelope:
 * { "message": "...", "errors": {...} }
 */
export function checkErrorEnvelope(res: http.Response, endpoint: string, expectedStatus: number): boolean {
  const checks: Record<string, (r: typeof res) => boolean> = {
    [`${endpoint}: status ${expectedStatus}`]: (r) => r.status === expectedStatus,
    [`${endpoint}: has message`]: (r) => {
      try {
        const body = JSON.parse(r.body as string);
        return typeof body.message === 'string';
      } catch {
        return false;
      }
    },
    [`${endpoint}: has errors object`]: (r) => {
      try {
        const body = JSON.parse(r.body as string);
        return body.errors && typeof body.errors === 'object';
      } catch {
        return false;
      }
    },
  };

  return check(res, checks);
}

/**
 * Check that a response is 401 Unauthorized (for unauthenticated requests).
 */
export function checkUnauthorized(res: http.Response, endpoint: string): boolean {
  return checkErrorEnvelope(res, endpoint, 401);
}

/**
 * Check pagination meta values are consistent.
 */
export function checkPaginationConsistency(res: http.Response, endpoint: string): boolean {
  return check(res, {
    [`${endpoint}: count matches data length`]: (r) => {
      try {
        const body = JSON.parse(r.body as string);
        return body.meta.pagination.count === body.data.length;
      } catch {
        return false;
      }
    },
    [`${endpoint}: total >= count`]: (r) => {
      try {
        const body = JSON.parse(r.body as string);
        return body.meta.pagination.total >= body.meta.pagination.count;
      } catch {
        return false;
      }
    },
    [`${endpoint}: current_page >= 1`]: (r) => {
      try {
        const body = JSON.parse(r.body as string);
        return body.meta.pagination.current_page >= 1;
      } catch {
        return false;
      }
    },
    [`${endpoint}: total_pages >= 1 when data present`]: (r) => {
      try {
        const body = JSON.parse(r.body as string);
        if (body.data.length === 0) return true;
        return body.meta.pagination.total_pages >= 1;
      } catch {
        return false;
      }
    },
  });
}

// ---------------------------------------------------------------------------
// Re-exports
// ---------------------------------------------------------------------------

export { http, check, sleep };
export { BASE_URL, BOOTSTRAP_KEY, DYNAMIC_FIELDS };
export { FireflyResource, FireflyListEnvelope, FireflySingleEnvelope };
