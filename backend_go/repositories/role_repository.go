package repositories

import (
	"frbktg/backend_go/models"

	"gorm.io/gorm"
)

type RoleRepository interface {
	CreateRole(role *models.Role) error
	GetRoles() ([]models.Role, error)
	GetRoleByID(roleID uint) (*models.Role, error)
	UpdateRole(role *models.Role) error
	DeleteRole(roleID uint) error

	CreatePermission(permission *models.Permission) error
	GetPermissions() ([]models.Permission, error)

	AddPermissionToRole(roleID, permissionID uint) error
	RemovePermissionFromRole(roleID, permissionID uint) error
	GetRolePermissions(roleID uint) ([]models.Permission, error)

	SetUserRole(userID, roleID uint) error
	GetUserRoles(userID uint) ([]models.Role, error)

	AddUserPermission(userPermission *models.UserPermission) error
	RemoveUserPermission(userID, permissionID uint) error
	GetUserPermissions(userID uint) ([]models.UserPermission, error)
}

type gormRoleRepository struct {
	db *gorm.DB
}

func NewRoleRepository(db *gorm.DB) RoleRepository {
	return &gormRoleRepository{db: db}
}

func (r *gormRoleRepository) CreateRole(role *models.Role) error {
	return r.db.Create(role).Error
}

func (r *gormRoleRepository) GetRoles() ([]models.Role, error) {
	var roles []models.Role
	err := r.db.Find(&roles).Error
	return roles, err
}

func (r *gormRoleRepository) GetRoleByID(roleID uint) (*models.Role, error) {
	var role models.Role
	err := r.db.First(&role, roleID).Error
	return &role, err
}

func (r *gormRoleRepository) UpdateRole(role *models.Role) error {
	return r.db.Save(role).Error
}

func (r *gormRoleRepository) DeleteRole(roleID uint) error {
	return r.db.Delete(&models.Role{}, roleID).Error
}

func (r *gormRoleRepository) CreatePermission(permission *models.Permission) error {
	return r.db.Create(permission).Error
}

func (r *gormRoleRepository) GetPermissions() ([]models.Permission, error) {
	var permissions []models.Permission
	err := r.db.Find(&permissions).Error
	return permissions, err
}

func (r *gormRoleRepository) AddPermissionToRole(roleID, permissionID uint) error {
	rp := models.RolePermission{RoleID: roleID, PermissionID: permissionID}
	return r.db.Create(&rp).Error
}

func (r *gormRoleRepository) RemovePermissionFromRole(roleID, permissionID uint) error {
	rp := models.RolePermission{RoleID: roleID, PermissionID: permissionID}
	return r.db.Delete(&rp).Error
}

func (r *gormRoleRepository) GetRolePermissions(roleID uint) ([]models.Permission, error) {
	var permissions []models.Permission
	err := r.db.Table("permissions").
		Joins("join role_permissions on permissions.id = role_permissions.permission_id").
		Where("role_permissions.role_id = ?", roleID).Find(&permissions).Error
	return permissions, err
}

func (r *gormRoleRepository) SetUserRole(userID, roleID uint) error {
	// First remove all other roles for this user
	if err := r.db.Where("user_id = ?", userID).Delete(&models.UserRole{}).Error; err != nil {
		return err
	}
	ur := models.UserRole{UserID: userID, RoleID: roleID}
	return r.db.Create(&ur).Error
}

func (r *gormRoleRepository) GetUserRoles(userID uint) ([]models.Role, error) {
	var roles []models.Role
	err := r.db.Table("roles").
		Joins("join user_roles on roles.id = user_roles.role_id").
		Where("user_roles.user_id = ?", userID).Find(&roles).Error
	return roles, err
}

func (r *gormRoleRepository) AddUserPermission(userPermission *models.UserPermission) error {
	return r.db.Save(userPermission).Error
}

func (r *gormRoleRepository) RemoveUserPermission(userID, permissionID uint) error {
	up := models.UserPermission{UserID: userID, PermissionID: permissionID}
	return r.db.Delete(&up).Error
}

func (r *gormRoleRepository) GetUserPermissions(userID uint) ([]models.UserPermission, error) {
	var userPermissions []models.UserPermission
	err := r.db.Where("user_id = ?", userID).Find(&userPermissions).Error
	return userPermissions, err
}
