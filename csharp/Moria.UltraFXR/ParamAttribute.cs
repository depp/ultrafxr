using System;

namespace Moria.UltraFXR
{
	[System.AttributeUsage(System.AttributeTargets.Property)]
	public abstract class ParamAttribute : Attribute
	{
		/// <summary>
		/// Gets the order in which the parameter should be presented to the user.
		/// </summary>
		public int Order { get; private set; }

		/// <summary>
		/// Gets the parameter name.
		/// </summary>
		public string Name { get; private set; }

		/// <summary>
		/// Gets the default value for the parameter.
		/// </summary>
		public abstract object Default { get; }

		public ParamAttribute(int order, string name)
		{
			this.Order = order;
			this.Name = name;
		}
	}
}
