const express = require('express');
const satteliteRoute = require('./sattelite.route');
const indexRoute = require('./v1/default.route');

const router = express.Router();

const routes = [
  {
    path: '/',
    route: indexRoute,
  },
  {
    path: '/api',
    route: satteliteRoute,
  },
];

routes.forEach(({ path, route }) => {
  router.use(path, route);
});

module.exports = router;
