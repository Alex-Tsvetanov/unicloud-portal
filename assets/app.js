// UniCloud Portal: the little JavaScript the pages need. Everything works without it as plain forms.
(function () {
  "use strict";

  // htmx sends the session's CSRF token on every state-changing request (ADR 0003)
  document.addEventListener("htmx:configRequest", function (e) {
    var meta = document.querySelector('meta[name="csrf-token"]');
    if (meta) e.detail.headers["X-CSRF-Token"] = meta.content;
  });

  // Staging page: forms post JSON to the staging API, then the users table reloads (htmx fragment)
  function reloadUsers() {
    if (window.htmx && document.getElementById("users-table")) {
      htmx.ajax("GET", "/staging/users-table", { target: "#users-table", swap: "outerHTML" });
    }
  }

  function lines(v) {
    return (v || "").split(/\r?\n/).map(function (s) { return s.trim(); }).filter(Boolean);
  }

  function body(form) {
    var f = new FormData(form);
    if (form.dataset.kind === "client") {
      return {
        clientId: f.get("clientId"), name: f.get("name"),
        redirectUris: lines(f.get("redirectUris")),
        postLogoutRedirectUris: lines(f.get("postLogoutRedirectUris")),
        directory: f.get("directory") === "on"
      };
    }
    var b = {
      email: f.get("email"), password: f.get("password"), firstName: f.get("firstName"),
      lastName: f.get("lastName"), role: f.get("role")
    };
    if (f.get("faculty")) b.faculty = f.get("faculty");
    if (b.role === "STUDENT") {
      b.student = {
        facultyNumber: f.get("facultyNumber"), specialty: f.get("specialty"),
        studyYear: Number(f.get("studyYear")), status: f.get("status")
      };
    }
    return b;
  }

  function show(form, ok, text) {
    var out = form.parentElement.querySelector(".result");
    if (!out) return;
    out.className = "result flash flash-" + (ok ? "ok" : "error");
    out.textContent = text;
  }

  document.addEventListener("submit", function (e) {
    var form = e.target;
    if (form.classList.contains("json-form")) {
      e.preventDefault();
      fetch(form.dataset.endpoint, {
        method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify(body(form))
      }).then(function (r) { return r.json().then(function (j) { return { ok: r.ok, j: j }; }); })
        .then(function (res) {
          if (!res.ok) {
            var fe = (res.j.fieldErrors || []).map(function (x) { return x.field + ": " + x.message; }).join("; ");
            show(form, false, res.j.message + (fe ? " (" + fe + ")" : ""));
            return;
          }
          if (form.dataset.kind === "client") {
            show(form, true, "Регистриран. client_id: " + res.j.clientId + "  тайна (показва се веднъж): " + res.j.clientSecret);
          } else {
            show(form, true, "Създаден потребител #" + res.j.id + " (" + res.j.email + "). Може да влезе веднага.");
            reloadUsers();
          }
        })
        .catch(function () { show(form, false, "Мрежова грешка. Опитайте отново."); });
    }
    if (form.classList.contains("perm-form")) {
      e.preventDefault();
      var perms = Array.prototype.map.call(form.querySelectorAll("input[name=permissions]:checked"), function (i) { return i.value; });
      fetch("/api/v1/staging/users/" + form.dataset.user + "/permissions", {
        method: "PUT", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ permissions: perms })
      }).then(reloadUsers);
    }
  });
})();
