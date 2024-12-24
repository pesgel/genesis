
-- 指令任务表
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
    `created_at`        datetime                                                        NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'create time',
    `updated_at`        datetime                                                        NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'update time',
    `deleted`           tinyint                                                         NOT NULL DEFAULT '0' COMMENT '是否删除，0-否，1-是',
    PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci COMMENT='节点信息表';
