const request = require('supertest');
const httpStatus = require('http-status');

const app = require('../../src/app');
const setupTestDB = require('../utils/setupTestDB');
const { MaintenanceRequest } = require('../../src/models');
const { maintenanceRequestOne, insertMaintenanceRequests } = require('../fixtures/maintenanceRequest.fixture');

setupTestDB();

describe('Maintenance Request routes', () => {
  describe('POST /servicerequest', () => {
    test('should return 201 and successfully create a new maintenance request given valid data', async () => {
      const res = await request(app).post('/servicerequest').send(maintenanceRequestOne).expect(httpStatus.CREATED);

      expect(res.body).toEqual({
        id: expect.anything(),
        buildingCode: maintenanceRequestOne.buildingCode,
        description: maintenanceRequestOne.description,
        createdBy: maintenanceRequestOne.createdBy,
        lastModifiedBy: maintenanceRequestOne.lastModifiedBy,
        createdDate: expect.anything(),
        currentStatus: 'Created',
        lastModifiedDate: expect.anything(),
      });

      const dbMaintenanceRequest = await MaintenanceRequest.findById(res.body.id);
      expect(dbMaintenanceRequest).toBeDefined();
      expect(dbMaintenanceRequest).toMatchObject({
        buildingCode: maintenanceRequestOne.buildingCode,
        description: maintenanceRequestOne.description,
        createdBy: maintenanceRequestOne.createdBy,
        lastModifiedBy: maintenanceRequestOne.lastModifiedBy,
        createdDate: expect.anything(),
        currentStatus: 'Created',
        lastModifiedDate: expect.anything(),
      });
    });
  });

  describe('GET /servicerequest', () => {
    test('should return a 200 with all service requests', async () => {
      await insertMaintenanceRequests([maintenanceRequestOne]);
      const res = await request(app).get('/servicerequest').send().expect(httpStatus.OK);
      expect(res.body).toHaveLength(1);
      expect(res.body[0]).toEqual({
        id: expect.anything(),
        buildingCode: maintenanceRequestOne.buildingCode,
        description: maintenanceRequestOne.description,
        createdBy: maintenanceRequestOne.createdBy,
        lastModifiedBy: maintenanceRequestOne.lastModifiedBy,
        createdDate: expect.anything(),
        currentStatus: 'Created',
        lastModifiedDate: expect.anything(),
      });
    });
  });

  // TODO these test suites should be written
  describe('GET /servicerequest/:id', () => {});

  describe('PATCH /servicerequest/:id', () => {});

  describe('DELETE /servicerequest/:id', () => {});
});
