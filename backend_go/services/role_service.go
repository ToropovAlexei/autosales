package services

import (
	"frbktg/backend_go/models"
	"frbktg/backend_go/repositories"
)

type RoleService interface {
	CreateRole(name string) (*models.Role, error)
	GetRoles() ([]models.Role, error)
	GetRole(roleID uint) (*models.Role, error)
	UpdateRole(roleID uint, name string) (*models.Role, error)
	DeleteRole(roleID uint) error

	GetPermissions() ([]models.Permission, error)

	AddPermissionToRole(roleID, permissionID uint) error
	RemovePermissionFromRole(roleID, permissionID uint) error
	GetRolePermissions(roleID uint) ([]models.Permission, error)

	SetUserRole(userID, roleID uint) error
	GetUserRoles(userID uint) ([]models.Role, error)

	AddUserPermission(userID, permissionID uint, effect string) (*models.UserPermission, error)
	RemoveUserPermission(userID, permissionID uint) error
	GetUserPermissions(userID uint) ([]models.UserPermission, error)

	GetUserFinalPermissions(userID uint) ([]string, error)
}

type roleService struct {
	roleRepo repositories.RoleRepository
}

func NewRoleService(roleRepo repositories.RoleRepository) RoleService {
	return &roleService{roleRepo: roleRepo}
}

func (s *roleService) CreateRole(name string) (*models.Role, error) {
	role := &models.Role{Name: name}
	err := s.roleRepo.CreateRole(role)
	return role, err
}

func (s *roleService) GetRoles() ([]models.Role, error) {
	return s.roleRepo.GetRoles()
}

func (s *roleService) GetRole(roleID uint) (*models.Role, error) {
	return s.roleRepo.GetRoleByID(roleID)
}

func (s *roleService) UpdateRole(roleID uint, name string) (*models.Role, error) {
	role, err := s.roleRepo.GetRoleByID(roleID)
	if err != nil {
		return nil, err
	}
	role.Name = name
	err = s.roleRepo.UpdateRole(role)
	return role, err
}

func (s *roleService) DeleteRole(roleID uint) error {
	return s.roleRepo.DeleteRole(roleID)
}

func (s *roleService) GetPermissions() ([]models.Permission, error) {
	return s.roleRepo.GetPermissions()
}

func (s *roleService) AddPermissionToRole(roleID, permissionID uint) error {
	return s.roleRepo.AddPermissionToRole(roleID, permissionID)
}

func (s *roleService) RemovePermissionFromRole(roleID, permissionID uint) error {
	return s.roleRepo.RemovePermissionFromRole(roleID, permissionID)
}

func (s *roleService) GetRolePermissions(roleID uint) ([]models.Permission, error) {
	return s.roleRepo.GetRolePermissions(roleID)
}

func (s *roleService) SetUserRole(userID, roleID uint) error {
	return s.roleRepo.SetUserRole(userID, roleID)
}

func (s *roleService) GetUserRoles(userID uint) ([]models.Role, error) {
	return s.roleRepo.GetUserRoles(userID)
}

func (s *roleService) AddUserPermission(userID, permissionID uint, effect string) (*models.UserPermission, error) {
	up := &models.UserPermission{UserID: userID, PermissionID: permissionID, Effect: effect}
	err := s.roleRepo.AddUserPermission(up)
	return up, err
}

func (s *roleService) RemoveUserPermission(userID, permissionID uint) error {
	return s.roleRepo.RemoveUserPermission(userID, permissionID)
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
