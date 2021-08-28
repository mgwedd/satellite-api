const express = require('express');

const satelliteRoute = require('./v1/satellite.route');
const indexRoute = require('./v1/default.route');

const router = express.Router();

const routes = [
  {
    path: '/',
    route: indexRoute,
  },
  {
    path: `/api/v1/satellite`,
    route: satelliteRoute,
  },
];

routes.forEach(({ path, route }) => {
  router.use(path, route);
});

module.exports = router;
