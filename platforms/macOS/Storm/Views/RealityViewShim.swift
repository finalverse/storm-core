#if os(macOS)
import SwiftUI
import RealityKit

/// Lightweight compatibility shim so the project builds on macOS where
/// `RealityView` is not available on macOS. Provide a thin wrapper that
/// exposes the underlying `Scene` so the rest of the code can share the same
/// API surface.
public typealias RealityViewScene = RealityKit.Scene
@available(*, deprecated, renamed: "RealityViewScene")
public typealias RealityViewContent = RealityViewScene

public struct RealityView<Content>: View {
    private let setup: (RealityViewScene) -> Void
    private let update: (RealityViewScene) -> Void

    public init(_ setup: @escaping (RealityViewScene) -> Void,
                update: @escaping (RealityViewScene) -> Void = { _ in }) {
        self.setup = setup
        self.update = update
    }

    public var body: some View {
        ARViewContainer(setup: setup, update: update)
    }

    private struct ARViewContainer: NSViewRepresentable {
        let setup: (RealityViewScene) -> Void
        let updateCallback: (RealityViewScene) -> Void

        init(setup: @escaping (RealityViewScene) -> Void,
             update: @escaping (RealityViewScene) -> Void) {
            self.setup = setup
            self.updateCallback = update
        }

        func makeNSView(context: Context) -> ARView {
            let view = ARView(frame: .zero)
            setup(view.scene)
            return view
        }

        func updateNSView(_ nsView: ARView, context: Context) {
            updateCallback(nsView.scene)
        }
    }
}
#endif
