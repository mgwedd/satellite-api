const express = require('express');

const router = express.Router();

// TODO split these routes out later when productionizing
router.route(['/', 'health', '/healthcheck', '/monitoring']).get((req, res) =>
  res.render('index', {
    title: 'The Service Request API',
    message:
      'Use the /servicerequest (GET/POST) and /servicerequest/:id (GET/PATCH/DELETE) resource paths to CRUD service requests.',
  })
);
// Note — I added the doc note to response here to avoid needing to create doc routes, which aren't needed for the coding challenge, but clearly the msg
// should be in a doc, not in this response for a production server
module.exports = router;
