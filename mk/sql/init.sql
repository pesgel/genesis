
-- 指令任务表
DROP TABLE IF EXISTS `instruct`;
CREATE TABLE `instruct`
(
    `id`        varchar(128)        NOT NULL COMMENT '主键',
    `data`      text          CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci  COMMENT '数据',
    `name`      varchar(128)  CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci  NOT NULL DEFAULT '' COMMENT '名称',
    `des`       varchar(1024) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci  NOT NULL DEFAULT '' COMMENT '描述',
    `created_by`        varchar(128) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci   NOT NULL DEFAULT '' COMMENT '创建人',
    `updated_by`        varchar(128) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci   NOT NULL DEFAULT '' COMMENT '更新人',
    `created_at`        datetime                                                        NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'create time',
    `updated_at`        datetime                                                        NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'update time',
    `deleted`           tinyint                                                         NOT NULL DEFAULT '0' COMMENT '是否删除，0-否，1-是',
    PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci COMMENT='指令任务表';

-- 用户信息表
DROP TABLE IF EXISTS `user`;
CREATE TABLE `user`
(
    `id`        varchar(128)        NOT NULL COMMENT '主键',
    `name`      varchar(128)  CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci  NOT NULL DEFAULT '' COMMENT '名称',
    `username`  varchar(128)  CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci  NOT NULL DEFAULT '' COMMENT '别名',
    `email`     varchar(128)  CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci  NOT NULL DEFAULT '' COMMENT '邮箱',
    `phone`     varchar(64)  CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci  NOT NULL DEFAULT '' COMMENT '电话',
    `password`     varchar(1024) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci  NOT NULL DEFAULT '' COMMENT '密码',
    `remark`       varchar(1024) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci  NOT NULL DEFAULT '' COMMENT '描述',
    `created_by`        varchar(128) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci   NOT NULL DEFAULT '' COMMENT '创建人',
    `updated_by`        varchar(128) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci   NOT NULL DEFAULT '' COMMENT '更新人',
    `created_at`        datetime                                                        NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'create time',
    `updated_at`        datetime                                                        NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'update time',
    `deleted`           tinyint                                                         NOT NULL DEFAULT '0' COMMENT '是否删除，0-否，1-是',
    PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci COMMENT='用户信息表';

-- 节点管理
DROP TABLE IF EXISTS `node`;
CREATE TABLE `node`
(
    `id`        varchar(128)        NOT NULL COMMENT '主键',
    `name`      varchar(64)  CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci  NOT NULL DEFAULT '' COMMENT '节点名称',
    `host`      varchar(64)  CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci  NOT NULL DEFAULT '' COMMENT '主机',
    `port`      int  NOT NULL DEFAULT '22' COMMENT '端口',
    `account`   varchar(128)  CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci  NOT NULL DEFAULT '' COMMENT '账户',
    `password`     varchar(128) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci  NOT NULL DEFAULT '' COMMENT '密码',
    `created_by`        varchar(128) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci   NOT NULL DEFAULT '' COMMENT '创建人',
    `updated_by`        varchar(128) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci   NOT NULL DEFAULT '' COMMENT '更新人',
    `remark`            varchar(1024) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci  NOT NULL DEFAULT '' COMMENT '描述',
    `created_at`        datetime                                                        NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'create time',
    `updated_at`        datetime                                                        NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'update time',
    `deleted`           tinyint                                                         NOT NULL DEFAULT '0' COMMENT '是否删除，0-否，1-是',
    PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci COMMENT='节点信息表';


-- 流程执行任务表
DROP TABLE IF EXISTS `execute_task`;
CREATE TABLE `execute_task`
(
    `id`             varchar(128)        NOT NULL COMMENT '主键',
    `instruct_id`    varchar(128)    CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci  NOT NULL DEFAULT '' COMMENT '流程ID',
    `name`  varchar(128)    CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci  NOT NULL DEFAULT '' COMMENT '任务名',
    `instruct_name`  varchar(128)    CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci  NOT NULL DEFAULT '' COMMENT '流程名快照',
    `node_id`        varchar(128)    CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci  NOT NULL DEFAULT '' COMMENT '节点ID',
    `node_name`      varchar(128)    CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci  NOT NULL DEFAULT '' COMMENT '节点名快照',
    `state`          int  NOT NULL DEFAULT '0' COMMENT '执行状态',
    `replaces`       text CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci COMMENT '替换参数',
    `remark`         varchar(1024) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci  NOT NULL DEFAULT '' COMMENT '描述',
    `created_by`     varchar(128) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci   NOT NULL DEFAULT '' COMMENT '创建人',
    `updated_by`     varchar(128) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci   NOT NULL DEFAULT '' COMMENT '更新人',
    `created_at`     datetime                                                        NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'create time',
    `updated_at`     datetime                                                        NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'update time',
    `deleted`        tinyint                                                         NOT NULL DEFAULT '0' COMMENT '是否删除，0-否，1-是',
    PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci COMMENT='流程执行任务表';


-- 资产表
DROP TABLE IF EXISTS `asset`;
CREATE TABLE `asset`
(
    `id`             varchar(64)        NOT NULL COMMENT '主键',
    `name`           varchar(128)    CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci  NOT NULL DEFAULT '' COMMENT '资产名',
    `org_id`         varchar(64)    CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci  NOT NULL DEFAULT '' COMMENT '所属组织/业务线',
    `address`        varchar(128)    CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci  NOT NULL DEFAULT '' COMMENT '资产唯一地址',
    `location`       varchar(128)    CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci  NOT NULL DEFAULT '' COMMENT '地理位置或机房',
    `alias_name`     varchar(128)    CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci  NOT NULL DEFAULT '' COMMENT '资产别名',
    `asset_type`     varchar(32)     CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci  NOT NULL DEFAULT '' COMMENT '资产类型',
    `address_type`   varchar(32)     CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci  NOT NULL DEFAULT 'ip' COMMENT '地址类型：ip/dns/url/dsn',
    `status`          int  NOT NULL DEFAULT '0' COMMENT '资产状态',
    `remark`         varchar(1024) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci  NOT NULL DEFAULT '' COMMENT '描述',
    `created_by`     varchar(128) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci   NOT NULL DEFAULT '' COMMENT '创建人',
    `updated_by`     varchar(128) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci   NOT NULL DEFAULT '' COMMENT '更新人',
    `created_at`     datetime                                                        NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'create time',
    `updated_at`     datetime                                                        NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'update time',
    `deleted`        tinyint                                                         NOT NULL DEFAULT '0' COMMENT '是否删除，0-否，1-是',
    PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci COMMENT='资产表';


-- 资产协议表
DROP TABLE IF EXISTS `asset_protocol`;
CREATE TABLE `asset_protocol`
(
    `id`             varchar(64)     NOT NULL COMMENT '主键',
    `asset_id`       varchar(64)    CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci  NOT NULL DEFAULT '' COMMENT '关联资产ID',
    `protocol`       varchar(32)     CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci  NOT NULL DEFAULT '' COMMENT '协议类型,如 ssh/rdp/vnc',
    `port`           int             NOT NULL DEFAULT 0 COMMENT '端口',
    `status`         int  NOT NULL DEFAULT '0' COMMENT '状态',
    `remark`         varchar(1024) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci  NOT NULL DEFAULT '' COMMENT '描述',
    `created_by`     varchar(128) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci   NOT NULL DEFAULT '' COMMENT '创建人',
    `updated_by`     varchar(128) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci   NOT NULL DEFAULT '' COMMENT '更新人',
    `created_at`     datetime                                                        NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'create time',
    `updated_at`     datetime                                                        NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'update time',
    `deleted`        tinyint                                                         NOT NULL DEFAULT '0' COMMENT '是否删除，0-否，1-是',
    PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci COMMENT='资产协议表';


-- 资产账号协议表
DROP TABLE IF EXISTS `asset_account`;
CREATE TABLE `asset_account`
(
    `id`                varchar(64)     NOT NULL COMMENT '主键',
    `principal`         varchar(32)     CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci  NOT NULL DEFAULT '' COMMENT '身份',
    `credential`        varchar(512)    CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci  NOT NULL DEFAULT '' COMMENT '凭证',
    `asset_id`          varchar(64)     CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci  NOT NULL DEFAULT '' COMMENT '关联资产ID',
    `auth_type`         varchar(32)     CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci  NOT NULL DEFAULT '' COMMENT '凭证类型',
    `address`        varchar(128)    CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci  NOT NULL DEFAULT '' COMMENT '资产地址',
    `protocol_id`    varchar(64)     CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci  NOT NULL DEFAULT '' COMMENT '协议ID',
    `protocol`       varchar(32)     CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci  NOT NULL DEFAULT '' COMMENT '协议类型,如 ssh/rdp/vnc',
    `port`           int             NOT NULL DEFAULT 0 COMMENT '端口',
    `status`         int  NOT NULL DEFAULT '0' COMMENT '状态',
    `remark`         varchar(1024) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci  NOT NULL DEFAULT '' COMMENT '描述',
    `created_by`     varchar(128) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci   NOT NULL DEFAULT '' COMMENT '创建人',
    `updated_by`     varchar(128) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci   NOT NULL DEFAULT '' COMMENT '更新人',
    `created_at`     datetime                                                        NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'create time',
    `updated_at`     datetime                                                        NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'update time',
    `deleted`        tinyint                                                         NOT NULL DEFAULT '0' COMMENT '是否删除，0-否，1-是',
    PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci COMMENT='资产账号协议表';
