package handlers

import (
	"frbktg/backend_go/apperrors"
	"frbktg/backend_go/responses"
	"frbktg/backend_go/services"
	"net/http"
	"strconv"

	"github.com/gin-gonic/gin"
)

type RoleHandler struct {
	roleService services.RoleService
}

func NewRoleHandler(roleService services.RoleService) *RoleHandler {
	return &RoleHandler{roleService: roleService}
}

type createRolePayload struct {
	Name string `json:"name" binding:"required"`
}

func (h *RoleHandler) CreateRoleHandler(c *gin.Context) {
	var json createRolePayload
	if err := c.ShouldBindJSON(&json); err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: err.Error()})
		return
	}

	role, err := h.roleService.CreateRole(c, json.Name)
	if err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusCreated, role)
}

func (h *RoleHandler) GetRolesHandler(c *gin.Context) {
	roles, err := h.roleService.GetRoles()
	if err != nil {
		c.Error(err)
		return
	}
	responses.SuccessResponse(c, http.StatusOK, roles)
}

func (h *RoleHandler) GetRoleHandler(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 32)
	if err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: "Invalid ID"})
		return
	}

	role, err := h.roleService.GetRole(uint(id))
	if err != nil {
		c.Error(err)
		return
	}
	responses.SuccessResponse(c, http.StatusOK, role)
}

func (h *RoleHandler) UpdateRoleHandler(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 32)
	if err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: "Invalid ID"})
		return
	}

	var json createRolePayload
	if err := c.ShouldBindJSON(&json); err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: err.Error()})
		return
	}

	role, err := h.roleService.UpdateRole(c, uint(id), json.Name)
	if err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusOK, role)
}

func (h *RoleHandler) DeleteRoleHandler(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 32)
	if err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: "Invalid ID"})
		return
	}

	if err := h.roleService.DeleteRole(c, uint(id)); err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusNoContent, nil)
}

func (h *RoleHandler) GetPermissionsHandler(c *gin.Context) {
	permissions, err := h.roleService.GetPermissions()
	if err != nil {
		c.Error(err)
		return
	}
	responses.SuccessResponse(c, http.StatusOK, permissions)
}

func (h *RoleHandler) GetRolePermissionsHandler(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 32)
	if err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: "Invalid ID"})
		return
	}

	permissions, err := h.roleService.GetRolePermissions(uint(id))
	if err != nil {
		c.Error(err)
		return
	}
	responses.SuccessResponse(c, http.StatusOK, permissions)
}

type rolePermissionPayload struct {
	PermissionID uint `json:"permission_id" binding:"required"`
}

func (h *RoleHandler) AddPermissionToRoleHandler(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 32)
	if err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: "Invalid ID"})
		return
	}

	var json rolePermissionPayload
	if err := c.ShouldBindJSON(&json); err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: err.Error()})
		return
	}

	if err := h.roleService.AddPermissionToRole(c, uint(id), json.PermissionID); err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusNoContent, nil)
}

func (h *RoleHandler) RemovePermissionFromRoleHandler(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 32)
	if err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: "Invalid ID"})
		return
	}

	permissionID, err := strconv.ParseUint(c.Param("permission_id"), 10, 32)
	if err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: "Invalid Permission ID"})
		return
	}

	if err := h.roleService.RemovePermissionFromRole(c, uint(id), uint(permissionID)); err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusNoContent, nil)
}

func (h *RoleHandler) GetUserRolesHandler(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 32)
	if err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: "Invalid ID"})
		return
	}

	roles, err := h.roleService.GetUserRoles(uint(id))
	if err != nil {
		c.Error(err)
		return
	}
	responses.SuccessResponse(c, http.StatusOK, roles)
}

type setUserRolePayload struct {
	RoleID uint `json:"role_id" binding:"required"`
}

func (h *RoleHandler) SetUserRoleHandler(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 32)
	if err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: "Invalid ID"})
		return
	}

	var json setUserRolePayload
	if err := c.ShouldBindJSON(&json); err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: err.Error()})
		return
	}

	if err := h.roleService.SetUserRole(c, uint(id), json.RoleID); err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusNoContent, nil)
}

func (h *RoleHandler) GetUserPermissionsHandler(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 32)
	if err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: "Invalid ID"})
		return
	}

	permissions, err := h.roleService.GetUserPermissions(uint(id))
	if err != nil {
		c.Error(err)
		return
	}
	responses.SuccessResponse(c, http.StatusOK, permissions)
}

type userPermissionPayload struct {
	PermissionID uint   `json:"permission_id" binding:"required"`
	Effect       string `json:"effect" binding:"required"`
}

func (h *RoleHandler) AddUserPermissionHandler(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 32)
	if err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: "Invalid ID"})
		return
	}

	var json userPermissionPayload
	if err := c.ShouldBindJSON(&json); err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: err.Error()})
		return
	}

	up, err := h.roleService.AddUserPermission(c, uint(id), json.PermissionID, json.Effect)
	if err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusCreated, up)
}

func (h *RoleHandler) RemoveUserPermissionHandler(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 32)
	if err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: "Invalid ID"})
		return
	}

	permissionID, err := strconv.ParseUint(c.Param("permission_id"), 10, 32)
	if err != nil {
		c.Error(&apperrors.ErrValidation{Base: apperrors.New(400, "", err), Message: "Invalid Permission ID"})
		return
	}

	if err := h.roleService.RemoveUserPermission(c, uint(id), uint(permissionID)); err != nil {
		c.Error(err)
		return
	}

	responses.SuccessResponse(c, http.StatusNoContent, nil)
}
