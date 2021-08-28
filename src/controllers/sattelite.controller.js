const httpStatus = require('http-status');
const ApiError = require('../utils/ApiError');
const catchAsync = require('../utils/catchAsync');
const { satteliteService } = require('../services');

const bulkCreateSattelites = catchAsync(async (req, res) => {
  const maintenanceRequest = await satteliteService.bulkCreateSattelites(req.body);
  res.status(httpStatus.CREATED).send(maintenanceRequest);
});

const listSattelites = catchAsync(async (req, res) => {
  // TODO Add filtering and pagination options in a production-ready version of this server so the API can scale
  const sattelitesList = await satteliteService.listSattelites();
  res.send(sattelitesList);
});

const getSattelite = catchAsync(async (req, res) => {
  const sattelite = await satteliteService.getSatteliteById(req.params.id);
  if (!sattelite) {
    throw new ApiError(httpStatus.NOT_FOUND, 'Sattelite not found');
  }
  res.send(sattelite);
});

const updateSattelite = catchAsync(async (req, res) => {
  const updatedSattelite = await satteliteService.updateSatteliteById(req.params.id, req.body);
  res.send(updatedSattelite);
});

const deleteSattelite = catchAsync(async (req, res) => {
  await satteliteService.deleteSatteliteById(req.params.id);
  res.status(httpStatus.NO_CONTENT).send();
});

const getNextVisible = catchAsync(async (req, res) => {
  res.send(httpStatus.NOT_IMPLEMENTED);
});

const getOverhead = catchAsync(async (req, res) => {
  res.send(httpStatus.NOT_IMPLEMENTED);
});

module.exports = {
  bulkCreateSattelites,
  listSattelites,
  getSattelite,
  updateSattelite,
  deleteSattelite,
  getNextVisible,
  getOverhead,
};
