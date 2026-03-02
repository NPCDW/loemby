# 数据同步

本应用需要同步的数据包括：
- **数据库文件**：存储应用的所有数据，包括所有能在页面上看到的数据与配置
- **配置文件**：存储应用的数据库连接、日志级别等设置，可以不同步，使用默认设置也可

## 推荐方案

### 方案一：使用云存储同步（数据库 + 配置）

使用坚果云、Dropbox 等实时同步工具同步整个应用数据目录：

**Windows 数据路径**
```
%AppData%\Roaming\com.loemby.app\
```

**Linux/macOS 数据路径**
```
~/.config/loemby/
```

**优点**
- 同步数据库和配置文件，迁移简单
- 无需额外搭建服务器

---

### 方案二：使用 PostgreSQL 远程数据库（仅数据库）

默认使用 SQLite 本地数据库，可通过配置切换为 PostgreSQL 远程数据库：

**配置示例**
```json
{
    "database_type": "postgres",
    "database_url": "postgres://username:password@127.0.0.1:5432/database_name"
}
```

**优点**
- 多设备实时数据同步
- 配置文件仍本地存储，可差异化设置

**注意**
- 需要保证与数据库良好的网络连接
- 需要自行搭建或租赁 PostgreSQL 服务器

