import { ElButton, ElNotification } from "element-plus";
import invokeApi from "../api/invokeApi";
import { h } from "vue";

async function getUpdate() {
    invokeApi.updater().then(res => {
        if (res) {
            ElNotification.success({
                title: '新版本准备就绪',
                message: h('p', null, [
                  h('span', null, '重启体验，现在重启？'),
                  h(ElButton, {
                    'size': 'small',
                    'type': 'success',
                    'click': () => {
                      invokeApi.restartApp()
                    },
                  }, "重启"),
                ]),
            })
        }
    })
}

export default {
    getUpdate
}