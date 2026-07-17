/**
 * AmllPlayer — AMLL PrebuiltLyricPlayer 的 React 根组件
 *
 * 接收一个外部创建的 Jotai store（由 Vue 桥接层 useAmllBridge 管理），
 * 仅负责渲染 <Provider store={store}><PrebuiltLyricPlayer /></Provider>。
 *
 * 状态同步由 Vue 侧的 watchers 完成：store.set(atom, value) → React 自动 re-render。
 */
import { Provider } from 'jotai'
import { PrebuiltLyricPlayer } from '@applemusic-like-lyrics/react-full'
import '@applemusic-like-lyrics/react-full/style.css'

/**
 * @param {{ store: import('jotai').Store }} props
 */
export function AmllPlayer({ store }) {
	return (
		<Provider store={store}>
			<PrebuiltLyricPlayer
				style={{
					width: '100%',
					height: '100%',
				}}
			/>
		</Provider>
	)
}

export default AmllPlayer
