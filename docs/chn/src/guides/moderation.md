审核
==========

`ord` 包含一个区块浏览器，你可以在本地使用命令 `ord server`开启。

区块浏览器允许查看铭文。 铭文是用户生成的内容，可能令人反感或违法。

运行ord区块浏览器实例的每个人都有责任了解他们对非法内容的责任，并决定适合他们实例的审核政策。

为了防止特定的铭文显示在`ord`实例上，它们可以包含在 YAML 配置文件中，该文件使用 `--config`选项加载。

要隐藏铭文，首先创建一个配置文件，其中包含要隐藏的铭文 ID：

```yaml
hidden:
- 0000000000000000000000000000000000000000000000000000000000000000i0
```

`ord` 配置文件的建议名称是 `ord.yaml`，但可以使用任何文件名。

然后将文件在服务启动的使用使用 `--config` :

`ord --config ord.yaml server`

请注意， `--config` 选项的位置在  `ord` 之后但是在  `server`子命令前。

`ord` 必须重启才可以加载在配置文件中的更改。

`ordinals.com`
--------------

 `ordinals.com` 实例使用 `systemd` 运行名为 `ord`的 `ord server` 服务，配置文件在 `/var/lib/ord/ord.yaml`.

要在 ordinals.com 上隐藏铭文:

1. 使用SSH登陆服务器
2. 在 `/var/lib/ord/ord.yaml`中增加铭文ID
3. 使用 `systemctl restart ord` 重启服务
4. 通过 `journalctl -u ord` 重启

目前，ord 重启速度较慢，因此站点不会立即恢复在线。
