package services

import (
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"

	"github.com/gin-gonic/gin"
)

type RoleService interface {
	CreateRole(ctx *gin.Context, name string) (*models.Role, error)
	GetRoles() ([]models.Role, error)
	GetRole(roleID uint) (*models.Role, error)
	UpdateRole(ctx *gin.Context, roleID uint, name string) (*models.Role, error)
	DeleteRole(ctx *gin.Context, roleID uint) error

	GetPermissions() ([]models.Permission, error)

	AddPermissionToRole(ctx *gin.Context, roleID, permissionID uint) error
	RemovePermissionFromRole(ctx *gin.Context, roleID, permissionID uint) error
	GetRolePermissions(roleID uint) ([]models.Permission, error)

	SetUserRole(ctx *gin.Context, userID, roleID uint) error
	GetUserRoles(userID uint) ([]models.Role, error)

	AddUserPermission(ctx *gin.Context, userID, permissionID uint, effect string) (*models.UserPermission, error)
	RemoveUserPermission(ctx *gin.Context, userID, permissionID uint) error
	GetUserPermissions(userID uint) ([]models.UserPermission, error)

	GetUserFinalPermissions(userID uint) ([]string, error)
}

type roleService struct {
	roleRepo      repositories.RoleRepository
	auditLogService AuditLogService
}

func NewRoleService(roleRepo repositories.RoleRepository, auditLogService AuditLogService) RoleService {
	return &roleService{roleRepo: roleRepo, auditLogService: auditLogService}
}

func (s *roleService) CreateRole(ctx *gin.Context, name string) (*models.Role, error) {
	role := &models.Role{Name: name}
	if err := s.roleRepo.CreateRole(role); err != nil {
		return nil, err
	}
	s.auditLogService.Log(ctx, "ROLE_CREATE", "Role", role.ID, map[string]interface{}{"after": role})
	return role, nil
}

func (s *roleService) GetRoles() ([]models.Role, error) {
	return s.roleRepo.GetRoles()
}

func (s *roleService) GetRole(roleID uint) (*models.Role, error) {
	return s.roleRepo.GetRoleByID(roleID)
}

func (s *roleService) UpdateRole(ctx *gin.Context, roleID uint, name string) (*models.Role, error) {
	before, err := s.roleRepo.GetRoleByID(roleID)
	if err != nil {
		return nil, err
	}

	after := *before // Create a copy
	after.Name = name

	if err := s.roleRepo.UpdateRole(&after); err != nil {
		return nil, err
	}
	s.auditLogService.Log(ctx, "ROLE_UPDATE", "Role", roleID, map[string]interface{}{"before": before, "after": after})
	return &after, nil
}

func (s *roleService) DeleteRole(ctx *gin.Context, roleID uint) error {
	before, err := s.roleRepo.GetRoleByID(roleID)
	if err != nil {
		return err // Or handle not found case
	}
	if err := s.roleRepo.DeleteRole(roleID); err != nil {
		return err
	}
	s.auditLogService.Log(ctx, "ROLE_DELETE", "Role", roleID, map[string]interface{}{"before": before})
	return nil
}

func (s *roleService) GetPermissions() ([]models.Permission, error) {
	return s.roleRepo.GetPermissions()
}

func (s *roleService) AddPermissionToRole(ctx *gin.Context, roleID, permissionID uint) error {
	if err := s.roleRepo.AddPermissionToRole(roleID, permissionID); err != nil {
		return err
	}
	s.auditLogService.Log(ctx, "ROLE_PERMISSION_ADD", "Role", roleID, map[string]interface{}{"permission_id": permissionID})
	return nil
}

func (s *roleService) RemovePermissionFromRole(ctx *gin.Context, roleID, permissionID uint) error {
	if err := s.roleRepo.RemovePermissionFromRole(roleID, permissionID); err != nil {
		return err
	}
	s.auditLogService.Log(ctx, "ROLE_PERMISSION_REMOVE", "Role", roleID, map[string]interface{}{"permission_id": permissionID})
	return nil
}

func (s *roleService) GetRolePermissions(roleID uint) ([]models.Permission, error) {
	return s.roleRepo.GetRolePermissions(roleID)
}

func (s *roleService) SetUserRole(ctx *gin.Context, userID, roleID uint) error {
	// For simplicity, logging the after state. A more complex implementation could log before and after.
	if err := s.roleRepo.SetUserRole(userID, roleID); err != nil {
		return err
	}
	s.auditLogService.Log(ctx, "USER_ROLE_SET", "User", userID, map[string]interface{}{"role_id": roleID})
	return nil
}

func (s *roleService) GetUserRoles(userID uint) ([]models.Role, error) {
	return s.roleRepo.GetUserRoles(userID)
}

func (s *roleService) AddUserPermission(ctx *gin.Context, userID, permissionID uint, effect string) (*models.UserPermission, error) {
	up := &models.UserPermission{UserID: userID, PermissionID: permissionID, Effect: effect}
	if err := s.roleRepo.AddUserPermission(up); err != nil {
		return nil, err
	}
	s.auditLogService.Log(ctx, "USER_PERMISSION_ADD", "User", userID, map[string]interface{}{"after": up})
	return up, nil
}

func (s *roleService) RemoveUserPermission(ctx *gin.Context, userID, permissionID uint) error {
	if err := s.roleRepo.RemoveUserPermission(userID, permissionID); err != nil {
		return err
	}
	s.auditLogService.Log(ctx, "USER_PERMISSION_REMOVE", "User", userID, map[string]interface{}{"permission_id": permissionID})
	return nil
}

func (s *roleService) GetUserPermissions(userID uint) ([]models.UserPermission, error) {
	return s.roleRepo.GetUserPermissions(userID)
}

func (s *roleService) GetUserFinalPermissions(userID uint) ([]string, error) {
	userRoles, err := s.roleRepo.GetUserRoles(userID)
	if err != nil {
		return nil, err
	}

	finalPermissions := make(map[string]bool)

	// Get all permissions once to use as a lookup table
	allPermissions, err := s.roleRepo.GetPermissions()
	if err != nil {
		return nil, err
	}
	permissionsMap := make(map[uint]models.Permission)
	for _, p := range allPermissions {
		permissionsMap[p.ID] = p
	}

	for _, role := range userRoles {
		if role.IsSuper {
			// Super admin has all permissions
			for _, p := range allPermissions {
				finalPermissions[p.Name] = true
			}
			// No need to check other roles or individual permissions
			break
		}

		rolePermissions, err := s.roleRepo.GetRolePermissions(role.ID)
		if err != nil {
			return nil, err
		}
		for _, p := range rolePermissions {
			finalPermissions[p.Name] = true
		}
	}

	userPermissions, err := s.roleRepo.GetUserPermissions(userID)
	if err != nil {
		return nil, err
	}

	for _, up := range userPermissions {
		if permission, ok := permissionsMap[up.PermissionID]; ok {
			if up.Effect == "allow" {
				finalPermissions[permission.Name] = true
			} else if up.Effect == "deny" {
				delete(finalPermissions, permission.Name)
			}
		}
	}

	// Convert map to slice
	var result []string
	for p := range finalPermissions {
		result = append(result, p)
	}

	return result, nil
}
