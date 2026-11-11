// Simulates the 08:00 enrollment rush against your deployed portal.
// Run: k6 run -e BASE_URL=https://<your-app-url> spike.js
// Java speeds up after its first minute of traffic, so do a short warm-up run first.
import http from "k6/http";
import { check, sleep } from "k6";

const BASE_URL = __ENV.BASE_URL;
if (!BASE_URL) {
  throw new Error("Set BASE_URL, e.g. k6 run -e BASE_URL=https://<your-app-url> spike.js");
}

export const options = {
  scenarios: {
    morning_rush: {
      executor: "ramping-vus",
      startVUs: 10,                          // the quiet night
      stages: [
        { duration: "30s", target: 10 },
        { duration: "30s", target: 500 },    // 08:00, everyone hits refresh (raise this in rehearsals until something breaks)
        { duration: "2m", target: 500 },
        { duration: "30s", target: 0 },
      ],
    },
  },
  // Pass/fail criteria that mirror your SLOs
  thresholds: {
    http_req_failed: ["rate<0.005"],         // at least 99.5% of requests succeed
    http_req_duration: ["p(95)<1000"],       // 95% of requests answer within 1 second
  },
};

export default function () {
  // Swap in the real flow once the API exists: log in -> profile -> catalog -> enroll
  const res = http.get(`${BASE_URL}/`);
  check(res, { "status is 200": (r) => r.status === 200 });
  sleep(1);
}
