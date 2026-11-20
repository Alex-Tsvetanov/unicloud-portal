// Starts Swagger UI on /api-docs. A file, not an inline script, so the CSP can forbid inline script.
window.addEventListener("load", function () {
  window.ui = SwaggerUIBundle({
    url: "/openapi.json",
    dom_id: "#swagger-ui",
    deepLinking: true,
    tryItOutEnabled: false,
    persistAuthorization: false,
    defaultModelsExpandDepth: 0,
    docExpansion: "list",
  });
});
