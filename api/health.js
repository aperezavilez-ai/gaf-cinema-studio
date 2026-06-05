const { MVP_VERSION } = require("../web/lib/connections");

module.exports = function handler(_req, res) {
  res.setHeader("Content-Type", "application/json");
  res.status(200).json({
    ok: true,
    service: "cinemastudio-web",
    version: MVP_VERSION,
    timestamp: new Date().toISOString(),
  });
};
