using System;

namespace Moria.UltraFXR
{
	public interface IGenerator
	{
		/// <summary>
		/// Gets the total length, in samples, of the sound generated, if known.
		/// </summary>
		int? Length { get; }

		/// <summary>
		/// Render the sound into a buffer.
		/// </summary>
		/// <param name="buffer">The buffer to hold the sound.</param>
		/// <returns>Whether the generator has finished producing sound.</returns>
		bool Render(float[] buffer);
	}
}

